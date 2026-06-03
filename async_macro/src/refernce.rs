// Input Function
async fn test_async_input_function<'a, 'b>(x : &'a u32, y : &'b u32) -> &'b u32{
    y
}

use core::marker::{PhantomData, PhantomPinned};
use core::mem::{MaybeUninit, ManuallyDrop, transmute};
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Poll};
use core::cell::UnsafeCell;

struct TestAsyncInputFunction<'a, 'b>{
    bytes : [MaybeUninit<u8>; TestAsyncInputFunction::<'static, 'static>::LAYOUT.size()],

    not_unpin : PhantomPinned,
    remove_sync_send_by_default : PhantomData::<UnsafeCell::<()>>,

    _x : PhantomData<&'a u32>,
    _y : PhantomData<&'b u32>,
}

impl<'a, 'b> TestAsyncInputFunction<'a, 'b>{
    pub fn new(x : &'a u32, y : &'b u32) -> Self{
        let bytes = Self::pollable_fn(MaybeUninit::new(x), MaybeUninit::new(y));
        // not transmute because transmute doesn't like generics
        let mut out : MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe{
            core::ptr::copy_nonoverlapping(core::ptr::addr_of!(bytes).cast(), out.as_mut_ptr(), 1);
            core::mem::forget(bytes);

            out.assume_init()   
        }
    }

    async fn pollable_fn(x : MaybeUninit<&'a u32>, y : MaybeUninit<&'b u32>) -> &'b u32{
        async fn test_async_input_function<'a, 'b>(x : &'a u32, y : &'b u32) -> &'b u32{
            y
        }

        unsafe{
            // doing this so pollable_fn itself can be called with unint values for type punning stuff
            test_async_input_function(x.assume_init(), y.assume_init()).await
        }
    }

    fn as_pollable(self : Pin<&mut Self>) -> Pin<&mut impl Future<Output = &'b u32>>{
        unsafe{
            let mut value = Self::pollable_fn(MaybeUninit::uninit(), MaybeUninit::uninit());

            #[allow(unused)]
            let mut typed_output  = core::ptr::from_mut(&mut value);
            typed_output = transmute(self);
            Pin::new_unchecked(&mut *typed_output)
        }
    }

    const LAYOUT : core::alloc::Layout = Self::get_layout_of_async_fn();
    const fn get_layout_of_async_fn() -> core::alloc::Layout{
        const fn decay_return_type<'a, 'b, Output, FutureType : Future<Output = Output>, FutureConstructor>(_ptr : ManuallyDrop<FutureConstructor>) -> *mut FutureType where 
            FutureConstructor : Fn(MaybeUninit<&'a u32>, MaybeUninit<&'b u32>) -> FutureType{
            core::ptr::null_mut()
        }
        
        
        let y = Self::pollable_fn;
        let null_ptr = decay_return_type::<&u32, _, _>(ManuallyDrop::new(y));
        let ptr = unsafe{&*null_ptr};
        core::alloc::Layout::for_value(ptr)
    }
}

impl<'a, 'b> Future for TestAsyncInputFunction<'a, 'b>{
    type Output = &'b u32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.as_pollable().poll(cx)
    }
}

impl<'a, 'b> Drop for TestAsyncInputFunction<'a, 'b>{
    // self is pinned in drop
    fn drop(&mut self) {
        unsafe{
            let this = Pin::new_unchecked(self);
            core::ptr::from_mut(this.get_unchecked_mut()).drop_in_place();
        }
    }
}


