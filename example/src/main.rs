#[async_macro::make_answer(size = 44, align = 4)]
pub async fn TestAState(_x : core::num::NonZero<u32>) -> u32{
    let x : [u32; 9] = unsafe{core::mem::zeroed()};

    YeildOnceLocal::default().await;

    x[0] + x[5] + x[7]
}

#[async_macro::make_answer(size = 2, align = 1)]
pub async fn TestbState(){
    YeildOnceLocal::default().await;
}


pub const fn empty_waker() -> core::task::Context::<'static>{
    let raw = core::task::Waker::noop();
    core::task::Context::from_waker(raw)
}

pub struct Yeild;
impl Future for Yeild{
    type Output = ();
    fn poll(self: core::pin::Pin<&mut Self>, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::task::Poll::Pending
    }
}

#[derive(Debug, Default)]
pub struct YeildOnceLocal(bool);
impl Future for YeildOnceLocal{
    type Output = ();
    fn poll(mut self: core::pin::Pin<&mut Self>, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if self.0{
            core::task::Poll::Ready(())
        }else{
            self.0 = true;
            core::task::Poll::Pending
        }
    }
}

#[pin_project::pin_project]
pub struct InFlightRequests{
    #[pin]
    pub request_1 : TestAState,

    #[pin]
    pub request_2 : TestbState,
}

impl InFlightRequests{
    pub fn new() -> Self{
        Self { request_1: TestAState::new(core::num::NonZero::new(2).unwrap()), request_2: TestbState::new() }
    }

    pub fn poll(self : core::pin::Pin::<&mut Self>){
        let this = self.project();

        let _result = this.request_1.poll(&mut empty_waker());
        let _result = this.request_2.poll(&mut empty_waker());

    }
}


// cargo +nightly miri run
pub fn main(){
    let requests = InFlightRequests::new();
    
    let mut pinned = core::pin::pin!(requests);
    pinned.as_mut().poll();
    pinned.poll();

}   

pub const SIZE : usize = 32;
pub const SIZES : usize = 1usize << 63;


#[repr(align(32))]
pub struct AlignType;



async fn test_func<'b, 'c>(_x : &'b u32, _y : &'c u32) -> &'c u32{
    YeildOnceLocal::default().await;
    &0
}

const fn get_layout_of_async_fn() -> core::alloc::Layout{
    const fn decay_return_type<'a, 'b, Output, FutureType : Future<Output = Output>, FutureConstructor>(_ptr : core::mem::ManuallyDrop<FutureConstructor>) -> *mut FutureType where FutureConstructor : Fn(&'a u32, &'b u32) -> FutureType{
        core::ptr::null_mut()
    }
    
    
    let y = test_func;
    let null_ptr = decay_return_type::<&u32, _, _>(core::mem::ManuallyDrop::new(y));
    let ptr = unsafe{&*null_ptr};
    core::alloc::Layout::for_value(ptr)
}

const X : core::alloc::Layout = get_layout_of_async_fn();

pub struct Testing{
    pub value : [u8; X.size()],
    pub marker : [AlignType; 0],
}