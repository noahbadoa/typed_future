This crate allows convenient "deanonymization" of an async function. This let's you store them in structs without type erasure.

# Example
```rust
#[typed_future::typed_future(Name = Foo, Send = true, Sync = true)]
async fn foo<'a>(x : &'a mut u32) -> &'a mut u32{
    x
}

fn bar(cx: &mut core::task::Context){
    let mut x = 0;
    let static_foo : Foo = Foo::new(&mut x);
    let pinned_static_foo = core::pin::pin!(static_foo);
    _ = core::future::Future::poll(pinned_static_foo, cx);
}
```

# Limitations
__Lifetimes in async function arguments cannot be eluded__

__Send and Sync cannot be auto-dervied__ (will throw a compiler error if send/sync are set but the underlying future is not send/sync; the converse is not true)

__Const Generics or Generic types cannot be used__

No top level recursion. (Can't use generated type inside of the function definition)

Error messages aren't amazing

# ```#[typed_futures(name = String, Send = bool, Sync = bool)]```
Annotates an async function. Creates a struct that implements
```rust
pub fn new(arg1 : type1, arg2 : type2, argn : typen) -> return_type;
core::future::Future;
```

The type generated will have the same name as the function passed in but can be overwritten with the name arg to macro.
The dervied type is not Send or Sync by default. You can set the Send/Sync args to the macro to safely attempt to derive those bounds.
You can also write the Send/Sync marker declaration on the type directly if you want to ignore the compiler.
