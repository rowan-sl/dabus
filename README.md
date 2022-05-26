# DABus

DABus is a multi-type aplication bus. It allows for you to have multiple
completely independant "Handlers" or "Bus Stops" that you can interact with
and can interact with eachother **without aknowlaging eachothers existance**.
it maintains all of rust's type saftey and guarentees, while being able to act
in a highly dynamic fasion, almost like something out of javascript, but with none of the downsides.

## Key Features

- Type-Erased: the central `DABus` structure does not need to know any of the types related to a handler, or any events it is processing
- Asynchronous: all handlers are async
- Thread-Safe: multithreaded async executers are fully supported
- Type-Safe: handlers and event calls are fully statically typed
- Convenient: API does not force you to go through inconvenient loopholes

## Limitations

- As preivously mentioned, it is asynchronous and thread-safe. unfortunatally, there is no way around this, as all types must be Sync and Send, and async is a core requirement of how the executor functions
- Because of all of the dynamic typing used internally, this relies heavliy on dynamic dispatch and thus suffers from its performance issues (dont worry, its not *slow*)
- For debugging, this implements logging using the `log` crate, but it is still rather confusing to debug. hopefully this will change soon with backtraces

## Usage

A event handler for `DABus` is a simple struct method, something like this:

```rust
use dabus::BusInterface;

#[derive(Debug)]
struct ExampleHandler;

impl ExampleHandler {
    async fn hello_world(&mut self, arguments: (), mut _interface: BusInterface) {
        /*
        here, arguments is the args passed to the event call,
        and _interface is a struct for communicating with the bus that invoked it

        warning! do NOT use BusInterface outside of the async handler it was passed to!
        it may seem like a good way of doing things, but IT WILL NOT WORK!!!
        */
        println!("Hello, World!");
    }
}
```

and then define the event it goes along with

```rust
//            the name     args  return type
dabus::event!(HELLO_EVENT, (),   ());
```

To convert this from a regular struct to an bus stop, implement `BusStop`

```rust
use dabus::{BusStop, EventRegister};

impl BusStop for HelloHandler {
    // this function provides a list of the handlers that this stop provides
    fn registered_handlers(h: EventRegister<Self>) -> EventRegister<Self> {
        //        event def    event function
        h.handler(HELLO_EVENT, Self::hello_world)
    }
}
```

and finally, to use this

```rust
use dabus::DABus;

#[tokio::main]
async fn main() {
    let mut bus = DABus::new();
    // create a new instance of HelloHandler, and pass it to the bus for useage
    bus.register(HelloHandler);
    //      the event     arguments (type from event def)
    bus.fire(PRINT_EVENT, "Hello, World!".to_string()).await.unwrap();
    // you should now see Hello, World! on your terminal!
}
```

## TODO's

- [ ] docs
- [ ] tests
- [ ] backtraces (do LATER, do logging NOW)
- [ ] examples **IMPORTANT**
- [x] proper error handling
- [ ] multi-handler events
- [ ] more complex event matching (allow handlers to consume an event, after looking at the arguments?)
- [x] nested handler calls
- [ ] error forwarding
