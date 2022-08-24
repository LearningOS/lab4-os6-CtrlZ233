
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};
use crate::mm::{
    PhysAddr,
    VirtAddr,
    frame_alloc,
    frame_dealloc,
    PhysPageNum,
    FrameTracker,
    PageTable,
    StepByOne,
    kernel_token,
};
use super::BlockDevice;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;

pub struct HalImpl;

impl Hal for HalImpl {
    fn dma_alloc(pages: usize) -> usize {
        virtio_dma_alloc(pages).0
    }

    fn dma_dealloc(paddr: usize, pages: usize) -> i32 {
        virtio_dma_dealloc( PhysAddr::from(paddr), pages)
    }

    fn phys_to_virt(paddr: usize) -> usize {
        virtio_phys_to_virt(PhysAddr::from(paddr)).0
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        virtio_virt_to_phys(VirtAddr::from(vaddr)).0
    }
}

#[allow(unused)]
const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, HalImpl>>);

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { 
        UPSafeCell::new(Vec::new())
    };
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0.exclusive_access()
        .read_block(block_id, buf)
        .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0.exclusive_access()
        .write_block(block_id, buf)
        .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            Self(UPSafeCell::new(VirtIOBlk::<HalImpl>::new(
                &mut *(VIRTIO0 as *mut VirtIOHeader)
            ).unwrap()))
        }
    }
}

#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let mut ppn_base = PhysPageNum(0);
    for i in 0..pages {
        let frame = frame_alloc().unwrap();
        if i == 0 { ppn_base = frame.ppn; }
        assert_eq!(frame.ppn.0, ppn_base.0 + i);
        QUEUE_FRAMES.exclusive_access().push(frame);
    }
    ppn_base.into()
}

#[no_mangle]
pub extern "C" fn virtio_dma_dealloc(pa: PhysAddr, pages: usize) -> i32 {
    let mut ppn_base: PhysPageNum = pa.into();
    for _ in 0..pages {
        frame_dealloc(ppn_base);
        ppn_base.step();
    }
    0
}

#[no_mangle]
pub extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0)
}

#[no_mangle]
pub extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PageTable::from_token(kernel_token()).translate_va(vaddr).unwrap()
}
