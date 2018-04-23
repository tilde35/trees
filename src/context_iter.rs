pub trait ContextIterator<Context> {
    type Item;

    fn next(&mut self, context: &Context) -> Option<Self::Item>;
}
