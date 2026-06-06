use crate::common::*;
use crate::{typed_future, core, std};
use std::vec::Vec;
use core::default::Default;

#[typed_future::typed_future(name = TestAsyncFn, send = false, sync = false)]
async fn test_async_fn<'a, 'b>(x : &'a mut u32, y : &'b mut u32) -> &'a mut u32{
    *y += *x;

    let ptr = core::ptr::null::<u8>();

    let mut must_drop: Vec<u32> = Vec::new();

    YeildOnceLocal::default().await;

    let _y = core::ptr::null::<u8>() == ptr;
    must_drop.push(32);

    x
}

#[test]
fn testing(){
    let mut x = 0;
    let mut y= 123;

    let value : TestAsyncFn = TestAsyncFn::new(&mut x, &mut y);
    let pinned = core::pin::pin!(value);

    _ = poll_empty(pinned);
}

#[typed_future::typed_future(name = TestSendSync, send = true, sync = true)]
async fn test_send_sync<'a>(x : &'a mut u32) -> &'a mut u32{


    let mut must_drop: Vec<u32> = Vec::new();

    YeildOnceLocal::default().await;
    
    must_drop.push(32);

    x 
}

#[test]
fn testing_2(){
    let mut x = 0;

    let value : TestSendSync = TestSendSync::new(&mut x);
    let pinned = core::pin::pin!(value);

    _ = poll_empty(pinned);
}
