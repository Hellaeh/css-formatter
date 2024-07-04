use ring_buffer::RingBuffer;

const SIZE: usize = 16;

#[test]
fn basic() {
	let mut ring = RingBuffer::<usize, SIZE>::new(0);
	ring.push(42);

	assert_eq!(ring[0], 0);
	assert_eq!(ring[1], 42);

	ring.go(2);

	// Write full circle
	for i in 0..SIZE - 1 {
		ring.push((i + 1) * 10);
	}

	assert!(ring.is_behind());

	assert_eq!(ring[0], 10);

	while ring.is_behind() {
		ring.go(1);
	}

	assert_eq!(ring[0], (14 + 1) * 10);
}
