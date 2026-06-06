// Documentation of what proc_macro should expand to use while writing proc_macro

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


use crate::core;
use core::default::Default;
use core::marker::{PhantomData, PhantomPinned};
use core::mem::{MaybeUninit, ManuallyDrop, transmute};
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Poll};
use core::cell::UnsafeCell;

pub struct TestAsyncInputFunction<'a, 'b>{
    bytes : [MaybeUninit<u8>; TestAsyncInputFunction::<'static, 'static>::LAYOUT.size()],

    not_unpin : PhantomPinned,
    remove_sync_send_by_default : PhantomData::<UnsafeCell::<()>>,

    alignment : TestAsyncInputFunctionAlignment_wskfexuazm::Align<{TestAsyncInputFunction::<'static, 'static>::LAYOUT.align()}>,

    _x : PhantomData<&'a u32>,
    _y : PhantomData<&'b u32>,
}

#[allow(non_snake_case)]
mod TestAsyncInputFunctionAlignment_wskfexuazm{
    // https://github.com/jswrenn/elain
    // can't use crate because proc-macro
    #[repr(transparent)]
    pub struct Align<const N: usize>([<Self as AlignedAssociated>::AlignmentStruct; 0]) where Self: AlignedAssociated;
    pub trait AlignedAssociated{
        type AlignmentStruct;
    }

    impl AlignedAssociated for Align<        1> { type AlignmentStruct = Align1;         }
    impl AlignedAssociated for Align<        2> { type AlignmentStruct = Align2;         }
    impl AlignedAssociated for Align<        4> { type AlignmentStruct = Align4;         }
    impl AlignedAssociated for Align<        8> { type AlignmentStruct = Align8;         }
    impl AlignedAssociated for Align<       16> { type AlignmentStruct = Align16;        }
    impl AlignedAssociated for Align<       32> { type AlignmentStruct = Align32;        }
    impl AlignedAssociated for Align<       64> { type AlignmentStruct = Align64;        }
    impl AlignedAssociated for Align<      128> { type AlignmentStruct = Align128;       }
    impl AlignedAssociated for Align<      256> { type AlignmentStruct = Align256;       }
    impl AlignedAssociated for Align<      512> { type AlignmentStruct = Align512;       }
    impl AlignedAssociated for Align<     1024> { type AlignmentStruct = Align1024;      }
    impl AlignedAssociated for Align<     2048> { type AlignmentStruct = Align2048;      }
    impl AlignedAssociated for Align<     4096> { type AlignmentStruct = Align4096;      }
    impl AlignedAssociated for Align<     8192> { type AlignmentStruct = Align8192;      }
    impl AlignedAssociated for Align<    16384> { type AlignmentStruct = Align16384;     }
    impl AlignedAssociated for Align<    32768> { type AlignmentStruct = Align32768;     }
    impl AlignedAssociated for Align<    65536> { type AlignmentStruct = Align65536;     }
    impl AlignedAssociated for Align<   131072> { type AlignmentStruct = Align131072;    }
    impl AlignedAssociated for Align<   262144> { type AlignmentStruct = Align262144;    }
    impl AlignedAssociated for Align<   524288> { type AlignmentStruct = Align524288;    }
    impl AlignedAssociated for Align<  1048576> { type AlignmentStruct = Align1048576;   }
    impl AlignedAssociated for Align<  2097152> { type AlignmentStruct = Align2097152;   }
    impl AlignedAssociated for Align<  4194304> { type AlignmentStruct = Align4194304;   }
    impl AlignedAssociated for Align<  8388608> { type AlignmentStruct = Align8388608;   }
    impl AlignedAssociated for Align< 16777216> { type AlignmentStruct = Align16777216;  }
    impl AlignedAssociated for Align< 33554432> { type AlignmentStruct = Align33554432;  }
    impl AlignedAssociated for Align< 67108864> { type AlignmentStruct = Align67108864;  }
    impl AlignedAssociated for Align<134217728> { type AlignmentStruct = Align134217728; }
    impl AlignedAssociated for Align<268435456> { type AlignmentStruct = Align268435456; }
    impl AlignedAssociated for Align<536870912> { type AlignmentStruct = Align536870912; }

    #[repr(align(        1))] pub struct Align1         ;
    #[repr(align(        2))] pub struct Align2         ;
    #[repr(align(        4))] pub struct Align4         ;
    #[repr(align(        8))] pub struct Align8         ;
    #[repr(align(       16))] pub struct Align16        ;
    #[repr(align(       32))] pub struct Align32        ;
    #[repr(align(       64))] pub struct Align64        ;
    #[repr(align(      128))] pub struct Align128       ;
    #[repr(align(      256))] pub struct Align256       ;
    #[repr(align(      512))] pub struct Align512       ;
    #[repr(align(     1024))] pub struct Align1024      ;
    #[repr(align(     2048))] pub struct Align2048      ;
    #[repr(align(     4096))] pub struct Align4096      ;
    #[repr(align(     8192))] pub struct Align8192      ;
    #[repr(align(    16384))] pub struct Align16384     ;
    #[repr(align(    32768))] pub struct Align32768     ;
    #[repr(align(    65536))] pub struct Align65536     ;
    #[repr(align(   131072))] pub struct Align131072    ;
    #[repr(align(   262144))] pub struct Align262144    ;
    #[repr(align(   524288))] pub struct Align524288    ;
    #[repr(align(  1048576))] pub struct Align1048576   ;
    #[repr(align(  2097152))] pub struct Align2097152   ;
    #[repr(align(  4194304))] pub struct Align4194304   ;
    #[repr(align(  8388608))] pub struct Align8388608   ;
    #[repr(align( 16777216))] pub struct Align16777216  ;
    #[repr(align( 33554432))] pub struct Align33554432  ;
    #[repr(align( 67108864))] pub struct Align67108864  ;
    #[repr(align(134217728))] pub struct Align134217728 ;
    #[repr(align(268435456))] pub struct Align268435456 ;
    #[repr(align(536870912))] pub struct Align536870912 ;
}   

impl<'a, 'b> TestAsyncInputFunction<'a, 'b>{
    pub fn new(x : &'a mut u32, y : &'b mut u32) -> Self{
        let bytes = Self::pollable_fn(MaybeUninit::new(x), MaybeUninit::new(y));

        let mut out : MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe{
            core::mem::transmute(out)
        }
    }

    // only exists to throw compiler error
    fn vaildate_trait_bounds(){
        fn is_sync<T : core::marker::Sync>(val : &T){}
        fn is_send<T : core::marker::Send>(val : &T){}

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
            FutureConstructor : core::ops::Fn(MaybeUninit<&'a mut u32>, MaybeUninit<&'b mut u32>) -> FutureType{
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

impl<'a, 'b> core::ops::Drop for TestAsyncInputFunction<'a, 'b>{
    // self is pinned in drop
    fn drop(&mut self) {
        unsafe{
            let this = Pin::new_unchecked(self).as_pollable();
            core::ptr::from_mut(this.get_unchecked_mut()).drop_in_place();
        }
    }
}


