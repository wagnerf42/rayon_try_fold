use crate::prelude::*;
use crate::Try;

pub struct Filter<I, F> {
    pub(crate) base: I,
    pub(crate) filter: F,
}

impl<I, F> ParallelIterator for Filter<I, F>
where
    I: ParallelIterator,
    F: Fn(&I::Item) -> bool + Send + Sync,
{
    type Item = I::Item;
    type Controlled = I::Controlled;
    type Enumerable = False;

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        let c = FilterConsumer {
            filter: &self.filter,
            base: consumer,
        };
        self.base.drive(c)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        return self.base.with_producer(Callback {
            callback,
            filter: self.filter,
        });
        struct Callback<CB, F> {
            callback: CB,
            filter: F,
        }
        impl<T, F, CB> ProducerCallback<T> for Callback<CB, F>
        where
            CB: ProducerCallback<T>,
            F: Fn(&T) -> bool + Sync,
        {
            type Output = CB::Output;
            fn call<P>(self, base: P) -> CB::Output
            where
                P: Producer<Item = T>,
            {
                let producer = FilterProducer {
                    base,
                    filter: &self.filter,
                };
                self.callback.call(producer)
            }
        }
    }
}

struct FilterProducer<'f, I, F> {
    base: I,
    filter: &'f F,
}

impl<'f, I, F> Iterator for FilterProducer<'f, I, F>
where
    I: Iterator,
    F: Fn(&I::Item) -> bool,
{
    type Item = I::Item;
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.base.size_hint();
        (0, max)
    }
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(elem) = self.base.next() {
            if (self.filter)(&elem) {
                return Some(elem);
            }
        }
        None
    }
}

impl<'f, I, F> DoubleEndedIterator for FilterProducer<'f, I, F>
where
    I: DoubleEndedIterator,
    F: Fn(&I::Item) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(elem) = self.base.next_back() {
            if (self.filter)(&elem) {
                return Some(elem);
            }
        }
        None
    }
}

impl<'f, I, F> Divisible for FilterProducer<'f, I, F>
where
    I: Producer,
    F: Fn(&I::Item) -> bool,
{
    type Controlled = I::Controlled;
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }

    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            FilterProducer {
                base: left,
                filter: self.filter,
            },
            FilterProducer {
                base: right,
                filter: self.filter,
            },
        )
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.divide_at(index);
        (
            FilterProducer {
                base: left,
                filter: self.filter,
            },
            FilterProducer {
                base: right,
                filter: self.filter,
            },
        )
    }
}

fn filter_fold<T, Acc>(
    predicate: impl Fn(&T) -> bool,
    fold: impl Fn(Acc, T) -> Acc,
) -> impl Fn(Acc, T) -> Acc {
    move |acc, item| {
        if predicate(&item) {
            fold(acc, item)
        } else {
            acc
        }
    }
}

fn filter_try_fold<'a, T, Acc, R: Try<Ok = Acc>>(
    predicate: &'a mut impl FnMut(&T) -> bool,
    mut fold: impl FnMut(Acc, T) -> R + 'a,
) -> impl FnMut(Acc, T) -> R + 'a {
    move |acc, item| {
        if predicate(&item) {
            fold(acc, item)
        } else {
            R::from_ok(acc)
        }
    }
}

impl<'f, I, F> Producer for FilterProducer<'f, I, F>
where
    I: Producer,
    F: Fn(&I::Item) -> bool + Sync,
{
    fn sizes(&self) -> (usize, Option<usize>) {
        self.base.sizes()
    }
    fn preview(&self, _: usize) -> Self::Item {
        panic!("FilterProducer is not previewable")
    }
    fn scheduler<'s, P: 's, R: 's>(&self) -> Box<dyn Scheduler<P, R> + 's>
    where
        P: Producer,
        P::Item: Send,
        R: Reducer<P::Item>,
    {
        self.base.scheduler()
    }
    fn partial_fold<B, FO>(&mut self, init: B, fold_op: FO, limit: usize) -> B
    where
        B: Send,
        FO: Fn(B, Self::Item) -> B,
    {
        let filter_op = self.filter;
        self.base
            .partial_fold(init, filter_fold(filter_op, fold_op), limit)
    }
    fn partial_try_fold<B, G, R>(&mut self, init: B, f: G, limit: usize) -> R
    where
        G: FnMut(B, Self::Item) -> R,
        R: Try<Ok = B>,
    {
        self.base
            .partial_try_fold(init, filter_try_fold(&mut self.filter, f), limit)
    }
    fn micro_block_sizes(&self) -> (usize, usize) {
        self.base.micro_block_sizes()
    }
}

pub struct FilterConsumer<'f, C, F> {
    pub(crate) base: C,
    pub(crate) filter: &'f F,
}

impl<'f, C: Clone, F> Clone for FilterConsumer<'f, C, F> {
    fn clone(&self) -> Self {
        FilterConsumer {
            base: self.base.clone(),
            filter: self.filter,
        }
    }
}

impl<'f, Item, F, C> Consumer<Item> for FilterConsumer<'f, C, F>
where
    F: Fn(&Item) -> bool + Send + Sync,
    C: Consumer<Item>,
{
    type Result = C::Result;
    type Reducer = C::Reducer;
    fn consume_producer<P>(self, producer: P) -> Self::Result
    where
        P: Producer<Item = Item>,
    {
        let filter_producer = FilterProducer {
            filter: self.filter,
            base: producer,
        };
        self.base.consume_producer(filter_producer)
    }
    fn to_reducer(self) -> Self::Reducer {
        self.base.to_reducer()
    }
}
