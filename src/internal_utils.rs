pub struct NodeKey {
    pub i: usize,
    pub j: usize,
}

impl core::fmt::Debug for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}]", self.i, self.j))
    }
}

pub struct DbgTree<'a, T, F>
where
    T: core::fmt::Debug,
    F: Fn(&'a [T], &mut dyn FnMut(usize, usize, &'a T)),
{
    nodes: &'a [T],
    visitor: F,
}

pub const fn as_dbg_tree<'a, T, F>(nodes: &'a [T], visitor: F) -> DbgTree<'a, T, F>
where
    T: core::fmt::Debug,
    F: Fn(&'a [T], &mut dyn FnMut(usize, usize, &'a T)),
{
    DbgTree { nodes, visitor }
}

impl<'a, T, F> core::fmt::Debug for DbgTree<'a, T, F>
where
    T: core::fmt::Debug,
    F: Fn(&'a [T], &mut dyn FnMut(usize, usize, &'a T)),
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = f.debug_map();
        let mut f = |i, j, value: &T| {
            formatter.entry(&NodeKey { i, j }, value);
        };
        (self.visitor)(self.nodes, &mut f);
        formatter.finish()
    }
}
