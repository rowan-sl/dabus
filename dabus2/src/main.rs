#[allow(unused_imports)]
#[macro_use]
extern crate log;

use dabus2::{EventDef, stop::BusStop};
use futures::future::BoxFuture;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
}

pub enum TestEvent {
    Hello((usize, String)),
}

static PRINT_EVENT: &'static EventDef<unique_type::new!(), String> = &unsafe { EventDef::new() };
static FLUSH_EVENT: &'static EventDef<unique_type::new!(), ()> = &unsafe { EventDef::new() };

#[derive(Debug)]
pub struct Printer {
    buffer: String,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            buffer: String::new()
        }
    }

    async fn print(&mut self, to_print: String) {
        self.buffer = format!("{}\n{}", self.buffer, to_print);
    }

    async fn flush(&mut self, _:()) {
        println!("{}", self.buffer);
    }
}

impl BusStop for Printer {
    fn registered_handlers(h: dabus2::event::Handlers<Self>) -> dabus2::event::Handlers<Self>
    where
            Self: Sized {
        h
            .handler(PRINT_EVENT, Self::print)
            .handler(FLUSH_EVENT, Self::flush)
    }
}

//https://gist.github.com/rust-play/4ec3fa20656b01d243fabe0f428bb77b
trait Handler<'a, H: 'a, At, Rt>: Fn(&'a mut H, At) -> Self::Fut {
    type Fut: ::futures::Future<Output = Rt> + Send + Sync + 'a;
}

impl<'a, H: 'a, At, Rt, F, Fut> Handler<'a, H, At, Rt> for F
where
    F: Fn(&'a mut H, At) -> Fut,
    Fut: ::futures::Future<Output = Rt> + Send + Sync + 'a,
{
    type Fut = Fut;
}

async fn handler<
    H: Send + 'static, //actually on a impl block in the real example
    At: ::core::fmt::Debug + Send + 'static,
    Rt: ::core::fmt::Debug + Sync + Send,
>(
    func: impl for<'a> Handler<'a, H, At, Rt>,
    receiver: &mut H,
    arg: At,
) -> Rt {
    let handler = RawHandler::new(func);
    (handler.real_fn)(receiver, arg).await
}

struct RawHandler<F> {
    pub(crate) real_fn: F,
}

impl<F> RawHandler<F> {
    fn new<H, At, Rt>(f: F) -> Self
    where
        F: for<'a> Handler<'a, H, At, Rt>,
    {
        Self { real_fn: f }
    }
}



