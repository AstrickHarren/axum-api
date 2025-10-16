use {
  axum_tracing_opentelemetry::tracing_opentelemetry_instrumentation_sdk,
  diesel::connection::Instrumentation,
  diesel_async::{AsyncConnectionCore, methods, return_futures},
  tracing::{Instrument, field::Empty, instrument::Instrumented},
};

pub(crate) struct OtelInstrument;
impl Instrumentation for OtelInstrument {
  fn on_connection_event(&mut self, event: diesel::connection::InstrumentationEvent<'_>) {
    use diesel::connection::InstrumentationEvent::*;
    match event {
      StartQuery { query, .. } => {
        let span = tracing::Span::current();
        span.record("db.query.text", query.to_string());
      }
      FinishQuery { query, error, .. } => {
        let span = tracing::Span::current();
        span.record("db.query.text", query.to_string());
        if let Some(error) = error {
          span.record("error.type", error.to_string());
        }
      }
      _ => (),
    }
  }
}

pub trait RunQueryDsl<Conn>: Sized {
  fn execute<'conn, 'query>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<Conn::ExecuteFuture<'conn, 'query>>
  where
    Conn: AsyncConnectionCore + Send,
    Self: methods::ExecuteDsl<Conn> + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::execute(self, conn).instrument(make_otel_span())
  }

  fn load<'query, 'conn, U>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<return_futures::LoadFuture<'conn, 'query, Self, Conn, U>>
  where
    U: Send,
    Conn: AsyncConnectionCore,
    Self: Send + methods::LoadQuery<'query, Conn, U> + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::load(self, conn).instrument(make_otel_span())
  }

  fn load_stream<'conn, 'query, U>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<Self::LoadFuture<'conn>>
  where
    Conn: AsyncConnectionCore,
    U: 'conn,
    Self: methods::LoadQuery<'query, Conn, U> + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::load_stream(self, conn).instrument(make_otel_span())
  }

  fn get_result<'query, 'conn, U>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<return_futures::GetResult<'conn, 'query, Self, Conn, U>>
  where
    U: Send + 'conn,
    Conn: AsyncConnectionCore,
    Self: methods::LoadQuery<'query, Conn, U> + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::get_result(self, conn).instrument(make_otel_span())
  }

  fn get_results<'query, 'conn, U>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<return_futures::LoadFuture<'conn, 'query, Self, Conn, U>>
  where
    U: Send,
    Conn: AsyncConnectionCore,
    Self: methods::LoadQuery<'query, Conn, U> + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::get_results(self, conn).instrument(make_otel_span())
  }

  fn first<'query, 'conn, U>(
    self,
    conn: &'conn mut Conn,
  ) -> Instrumented<return_futures::GetResult<'conn, 'query, diesel::dsl::Limit<Self>, Conn, U>>
  where
    U: Send + 'conn,
    Conn: AsyncConnectionCore,
    Self: diesel::query_dsl::methods::LimitDsl,
    diesel::dsl::Limit<Self>: methods::LoadQuery<'query, Conn, U> + Send + 'query,
  {
    <Self as diesel_async::RunQueryDsl<Conn>>::first(self, conn).instrument(make_otel_span())
  }
}

fn make_otel_span() -> tracing::Span {
  tracing_opentelemetry_instrumentation_sdk::otel_trace_span!(
    "Diesel SQL",
    "error.type" = Empty,
    db.system = "postgresql",
    otel.kind = "CLIENT",
    db.query.text = Empty,
    otel.status_code = Empty
  )
}

impl<T, Conn> RunQueryDsl<Conn> for T {}
