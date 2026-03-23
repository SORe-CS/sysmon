pub struct RingBuffer<T> {
    buf: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self where T: Clone {
        RingBuffer {
            buf: vec![None; capacity],
            capacity,
            head: 0,
            len: 0,
        }
    }


    pub fn push (&mut self, value: T) {
        self.buf[self.head] = Some(value);
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len +=1;
        }
    }

    pub fn latest(&self, n: usize) -> Vec<&T> {
        let n = n.min(self.len);
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let index = (self.head + self.capacity - n + i) % self.capacity;
            if let Some(val) = &self.buf[index] {
                result.push(val);
            }
        }
        result
    }

    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len == 0 }
}


#[cfg(test)]
mod tests {
    use super::*;

    //1. HAPPY PATH
    //new buffer should be empty with len 0
    #[test]
    fn new_buffer_is_empty() {
        //Arrange
        let rb: RingBuffer<f32> = RingBuffer::new(4);

        //Assert
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
    }

    //2. HAPPY PATH
    //pushing one value should make len 1 and is_empty false
    #[test]
    fn push_single_value() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);

        //Act
        rb.push(10.0);

        //Assert
        assert!(!rb.is_empty());
        assert_eq!(rb.len(), 1);

    }

    //3. BOUNDARY
    //pushing exactly capacity values should fill it
    //len should equal capacity, nothing overwritten
    #[test]
    fn fill_to_capacity() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);

        //Act
        rb.push(1.0);
        rb.push(2.0);
        rb.push(3.0);
        rb.push(4.0);

        //Assert
        assert_eq!(rb.len(),4);
        assert!(!rb.is_empty());

        let vals = rb.latest(4);
        assert_eq!(vals.len(),4);
        assert_eq!(*vals[0], 1.0);
        assert_eq!(*vals[3], 4.0);
    }

    //4. BOUNDARY [important]
    //pushing one past capacity should overwrite oldest
    //len should stay at capacity (not grow)
    //latest(capacity) should return newest values
    #[test]
    fn overwrite_oldest_when_full() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);
        rb.push(1.0);
        rb.push(2.0);
        rb.push(3.0);
        rb.push(4.0);

        //Act
        rb.push(5.0);

        //Assert
        assert_eq!(rb.len(), 4);

        let vals = rb.latest(4);
        assert_eq!(*vals[0], 2.0);
        assert_eq!(*vals[3], 5.0);
    }

    //5. EDGE CASE
    //latest(0) should always return empty vec 
    #[test]
    fn latest_zero_returns_empty() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);
        rb.push(1.0);
        rb.push(2.0);

        //Act
        let vals = rb.latest(0);

        //Assert
        assert!(vals.is_empty());
    }

    //6. EDGE CASE
    //latest(n) where n > len should return all available values
    //not panic, not return garbage
    #[test]
    fn latest_larger_than_len_clamps() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);
        rb.push(1.0);
        rb.push(2.0);

        //Act
        let vals = rb.latest(10);

        //Assert
        assert_eq!(vals.len(), 2);
        assert_eq!(*vals[0], 1.0);
        assert_eq!(*vals[1], 2.0);
    }

    //7. EDGE CASE
    //capacity of 1 - every push overwrites the only slot
    //latest(1) should always return the most recent value
    #[test]
    fn capacity_one_always_holds_latest() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(1);
        //Act
        rb.push(1.0);
        rb.push(2.0);
        rb.push(3.0);

        //Assert
        assert_eq!(rb.len(), 1);
        let vals = rb.latest(1);
        assert_eq!(*vals[0], 3.0);
    }

    //8. INVARIANT
    //len should never exceed capacity no matter how many pushed
    #[test]
    fn len_never_exceeds_capacity() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);
        //Act
        for i in 0..100 { 
            rb.push(i as f32)
        }

        //Assert
        assert_eq!(rb.len(), 4);
    }

    //9. ORDERING
    //latest() must return values in chronological order
    //oldest first, newest last
    #[test]
    fn latest_returns_chronological_order() {
        //Arrange
        let mut rb: RingBuffer<f32> = RingBuffer::new(4);
        rb.push(1.0);
        rb.push(2.0);
        rb.push(3.0);
        rb.push(4.0);
        rb.push(5.0);

        //Act
        let vals = rb.latest(3);

        //Assert
        assert_eq!(vals.len(), 3);
        assert_eq!(*vals[0], 3.0);
        assert_eq!(*vals[1], 4.0);
        assert_eq!(*vals[2], 5.0);
    }

}