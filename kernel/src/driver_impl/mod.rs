mod virtio_blk;

use alloc::sync::Arc;
use lazy_static::*;
type BlockDeviceImpl = virtio_blk::VirtIOBlock;

pub use drivers::BlockDevice;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i, &write_buffer);
        block_device.read_block(i, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    log::info!("block device test passed!");
}
