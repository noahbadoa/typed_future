/*
Restriction:
    lifetimes cannot be eluded at all
    Send + Sync cannot be autoderived (compile time checked but not compile time derived)
    Alingment must be manually specify (compile time checked but not compile time derived)
    No const Generics or Type Generics (Not way to get size Self::SIZE fails to compile)
*/ 

async fn test_async_input_function<'a, 'b>(x : &'a mut u32, y : &'b mut u32) -> &'a mut u32{
    *y += *x;

    let ptr = core::ptr::null::<u8>();

    YeildOnceLocal::default().await;

    let y = core::ptr::null::<u8>() == ptr;

    x
}



#[derive(Debug, Default)]
pub struct YeildOnceLocal(bool);
impl Future for YeildOnceLocal{
    type Output = ();
    fn poll(mut self: core::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0{
            Poll::Ready(())
        }else{
            self.0 = true;
            Poll::Pending
        }
    }
}



use core::marker::{PhantomData, PhantomPinned};
use core::mem::{MaybeUninit, ManuallyDrop, transmute};
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Poll};
use core::cell::UnsafeCell;

#[repr(align(8))]
pub struct TestAsyncInputFunction<'a, 'b>{
    bytes : [MaybeUninit<u8>; TestAsyncInputFunction::<'static, 'static>::LAYOUT.size()],

    not_unpin : PhantomPinned,
    remove_sync_send_by_default : PhantomData::<UnsafeCell::<()>>,

    _x : PhantomData<&'a u32>,
    _y : PhantomData<&'b u32>,
}

impl<'a, 'b> TestAsyncInputFunction<'a, 'b>{

    const _COMPLILER_ERROR : () = const{
        if Self::LAYOUT.align() != core::alloc::Layout::new::<Self>().align(){
            pub const NEEDED_SIZE : usize = 100;
            let mut val = StackStringBuffer::<NEEDED_SIZE>::new();
            val.insert_back("Future has alignment of ");
            val.insert_back(StackStringBuffer::<MAX_DECIMAL_WIDTH_U32>::from_u32( Self::LAYOUT.align() as u32).as_str());
            val.insert_back(" but declared alignment was ");
            val.insert_back(StackStringBuffer::<MAX_DECIMAL_WIDTH_U32>::from_u32( core::alloc::Layout::new::<Self>().align() as u32).as_str());

            panic!("{}", val.as_str());
        }

        // not need to check size always equal

        // ignore rest of this block just const string formating stuff
        pub struct StackStringBuffer<const MAX_SIZE : usize>{
            pub bytes : [u8; MAX_SIZE],
            pub length : usize
        }

        pub const MAX_DECIMAL_WIDTH_U32 : usize = 10;
        impl<const MAX_SIZE : usize> StackStringBuffer<MAX_SIZE>{
            pub const fn new() -> Self{
                Self { bytes: [0; MAX_SIZE], length: 0 }
            }

            pub const fn as_str<'a>(&'a self) -> &'a str{
                unsafe{
                    let slice = core::slice::from_raw_parts(core::ptr::from_ref(&self.bytes).cast::<u8>(), self.length);
                    match core::str::from_utf8(slice){
                        Ok(str) => str,
                        _ => panic!()
                    }
                }
            }

            pub const fn insert_back(&mut self, other : &str){
                unsafe{
                    let dst = core::ptr::from_mut(&mut self.bytes).cast::<u8>().add(self.length);
                    core::ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len());
                    self.length += other.len();
                };
            }

            pub const fn from_u32(val : u32) -> StackStringBuffer::<MAX_SIZE>{
                let mut bytes : [u8; MAX_SIZE] = [0; MAX_SIZE];
                let mut val = val;
                let mut counter = 0;

                while counter < bytes.len() {
                    bytes[counter] = b'0' + (val % 10) as u8;

                    val /= 10;
                    counter += 1;
                    if val == 0 {break;}
                }

                StackStringBuffer::<MAX_SIZE> { bytes, length: counter }
            }
        }
    };

    pub fn new(x : &'a mut u32, y : &'b mut u32) -> Self{
        let bytes = Self::pollable_fn(MaybeUninit::new(x), MaybeUninit::new(y));

        let mut out : MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe{
            core::mem::transmute(out)
        }
    }

    // only exists to through compiler error
    fn vaildate_trait_bounds(){
        fn is_sync<T : Sync>(val : &T){}
        fn is_send<T : Send>(val : &T){}

        let value = Self::pollable_fn(MaybeUninit::uninit(), MaybeUninit::uninit());
        is_sync(&value);
        is_send(&value);
    }

    async fn pollable_fn(x : MaybeUninit<&'a mut u32>, y : MaybeUninit<&'b mut u32>) -> &'a mut u32{
        async fn test_async_input_function<'a, 'b>(x : &'a mut u32, y : &'b mut u32) -> &'a mut u32{
            *y += *x;

            // let ptr = core::ptr::null::<u8>();

            YeildOnceLocal::default().await;

            // let y = core::ptr::null::<u8>() == ptr;

            x
        }

        unsafe{
            // doing this so pollable_fn itself can be called with unint values for type punning stuff
            test_async_input_function(x.assume_init(), y.assume_init()).await
        }
    }

    fn as_pollable(self : Pin<&mut Self>) -> Pin<&mut (impl Future<Output = &'a mut u32>  + use<'a, 'b>)>{
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
        const fn decay_return_type<'a, 'b, Output, FutureType : Future<Output = Output>, FutureConstructor>(_ptr : ManuallyDrop<FutureConstructor>) -> core::alloc::Layout where 
            FutureConstructor : Fn(MaybeUninit<&'a mut u32>, MaybeUninit<&'b mut u32>) -> FutureType{
            core::alloc::Layout::new::<FutureType>()
        }
        
        
        let y = Self::pollable_fn;
        decay_return_type::<&mut u32, _, _>(ManuallyDrop::new(y))
    }
}

impl<'a, 'b> Future for TestAsyncInputFunction<'a, 'b>{
    type Output = &'a mut u32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.as_pollable().poll(cx)
    }
}

impl<'a, 'b> Drop for TestAsyncInputFunction<'a, 'b>{
    // self is pinned in drop
    fn drop(&mut self) {
        unsafe{
            let this = Pin::new_unchecked(self).as_pollable();
            core::ptr::from_mut(this.get_unchecked_mut()).drop_in_place();
        }
    }
}


