use std::collections::VecDeque;

/// 一个支持O(1)时间获取最大值的队列
pub struct MaxQueue<T>
where
    T: PartialOrd + Copy,
{
    // 主队列，存储所有元素
    queue: VecDeque<T>,
    // 辅助队列，维护单调递减序列，用于快速获取最大值
    max_queue: VecDeque<T>,
}

impl<T> MaxQueue<T>
where
    T: PartialOrd + Copy,
{
    /// 创建一个新的空队列
    pub fn new() -> Self {
        MaxQueue {
            queue: VecDeque::new(),
            max_queue: VecDeque::new(),
        }
    }

    /// 将元素加入队列
    /// 时间复杂度：均摊O(1)
    pub fn enqueue(&mut self, value: T) {
        // 向主队列添加元素
        self.queue.push_back(value);

        // 维护辅助队列，确保单调递减
        // 从队尾移除所有小于当前值的元素
        while let Some(&last) = self.max_queue.back() {
            if last < value {
                self.max_queue.pop_back();
            } else {
                break;
            }
        }
        // 将当前值添加到辅助队列
        self.max_queue.push_back(value);
    }

    /// 移除并返回队列头部元素
    /// 时间复杂度：O(1)
    pub fn dequeue(&mut self) -> Option<T> {
        // 从主队列移除头部元素
        let front = self.queue.pop_front()?;

        // 如果移除的是最大值，也从辅助队列移除
        if let Some(&max_front) = self.max_queue.front() {
            if front == max_front {
                self.max_queue.pop_front();
            }
        }

        Some(front)
    }

    /// 返回队列头部元素但不移除
    /// 时间复杂度：O(1)
    pub fn peek(&self) -> Option<&T> {
        self.queue.front()
    }

    /// 返回队列中的最大值
    /// 时间复杂度：O(1)
    pub fn getmax(&self) -> Option<&T> {
        self.max_queue.front()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// 返回队列中的元素个数
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = MaxQueue::new();
        queue.enqueue(3);
        queue.enqueue(1);
        queue.enqueue(4);
        queue.enqueue(2);

        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(4));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_peek() {
        let mut queue = MaxQueue::new();
        queue.enqueue(5);
        queue.enqueue(3);

        assert_eq!(queue.peek(), Some(&5));
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_getmax() {
        let mut queue = MaxQueue::new();
        queue.enqueue(3);
        assert_eq!(queue.getmax(), Some(&3));

        queue.enqueue(1);
        assert_eq!(queue.getmax(), Some(&3));

        queue.enqueue(4);
        assert_eq!(queue.getmax(), Some(&4));

        queue.enqueue(2);
        assert_eq!(queue.getmax(), Some(&4));

        queue.dequeue(); // 移除3
        assert_eq!(queue.getmax(), Some(&4));

        queue.dequeue(); // 移除1
        assert_eq!(queue.getmax(), Some(&4));

        queue.dequeue(); // 移除4
        assert_eq!(queue.getmax(), Some(&2));

        queue.dequeue(); // 移除2
        assert_eq!(queue.getmax(), None);
    }

    #[test]
    fn test_empty_queue() {
        let mut queue = MaxQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.dequeue(), None);
        assert_eq!(queue.peek(), None);
        assert_eq!(queue.getmax(), None);
    }

    #[test]
    fn test_monotonic_sequence() {
        let mut queue = MaxQueue::new();
        
        // 递增序列
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        assert_eq!(queue.getmax(), Some(&4));
        
        // 递减序列
        queue.dequeue();
        queue.dequeue();
        queue.dequeue();
        queue.dequeue();
        
        queue.enqueue(4);
        queue.enqueue(3);
        queue.enqueue(2);
        queue.enqueue(1);
        assert_eq!(queue.getmax(), Some(&4));
        
        queue.dequeue(); // 移除4
        assert_eq!(queue.getmax(), Some(&3));
        
        queue.dequeue(); // 移除3
        assert_eq!(queue.getmax(), Some(&2));
    }
}

fn main() {
    // 示例用法
    let mut queue = MaxQueue::new();
    
    queue.enqueue(3);
    queue.enqueue(1);
    queue.enqueue(4);
    queue.enqueue(2);
    
    println!("队列中的最大值: {:?}", queue.getmax()); // 输出 Some(4)
    println!("队头元素: {:?}", queue.peek()); // 输出 Some(3)
    
    queue.dequeue(); // 移除3
    println!("移除队头后，最大值: {:?}", queue.getmax()); // 输出 Some(4)
    
    queue.dequeue(); // 移除1
    queue.dequeue(); // 移除4
    println!("移除两个元素后，最大值: {:?}", queue.getmax()); // 输出 Some(2)
}