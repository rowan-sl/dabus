#[allow(unused_imports)]
#[macro_use]
extern crate log;

use dabus2::EventDef;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
}

pub enum TestEvent {
    Hello((usize, String))
}

static TEST_EVENT: &'static EventDef<unique_type::new!(), ()> = &EventDef::new();

pub fn fire<Tag: unique_type::Unique, At, Rt>(def: &'static EventDef<Tag, At, Rt>, args: At) -> Rt {
    todo!()
}
