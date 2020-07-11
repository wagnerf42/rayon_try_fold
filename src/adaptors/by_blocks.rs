use crate::prelude::*;
use crate::schedulers::ByBlocksScheduler;

pub struct ByBlocks<I> {
    pub(crate) base: I,
}

// producer
impl<I: Iterator> Iterator for ByBlocks<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}

impl<P: Producer> Divisible for ByBlocks<P> {
    type Controlled = P::Controlled;
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (ByBlocks { base: left }, ByBlocks { base: right })
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.divide_at(index);
        (ByBlocks { base: left }, ByBlocks { base: right })
    }
}

impl<Q: Producer> Producer for ByBlocks<Q> {
    fn sizes(&self) -> (usize, Option<usize>) {
        self.base.sizes()
    }
    fn preview(&self, index: usize) -> Self::Item {
        self.base.preview(index)
    }
    fn scheduler<'s, P: 's, R: 's>(&self) -> Box<dyn Scheduler<P, R> + 's>
    where
        P: Producer,
        P::Item: Send,
        R: Reducer<P::Item>,
    {
        Box::new(ByBlocksScheduler {
            inner_scheduler: self.base.scheduler(),
        })
    }
}

// consumer
impl<C: Clone> Clone for ByBlocks<C> {
    fn clone(&self) -> Self {
        ByBlocks {
            base: self.base.clone(),
        }
    }
}

impl<Item, C: Consumer<Item>> Consumer<Item> for ByBlocks<C> {
    type Result = C::Result;
    type Reducer = C::Reducer;
    fn consume_producer<P>(self, producer: P) -> Self::Result
    where
        P: Producer<Item = Item>,
    {
        let producer = ByBlocks { base: producer };
        self.base.consume_producer(producer)
    }
    fn to_reducer(self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

// iterator

impl<I> ParallelIterator for ByBlocks<I>
where
    I: ParallelIterator,
{
    type Item = I::Item;
    type Controlled = I::Controlled;
    type Enumerable = False;
    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        let consumer = ByBlocks { base: consumer };
        self.base.drive(consumer)
    }
    fn with_producer<CB>(self, _callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        panic!("scheduling policies must be called as a consumer")
    }
}
