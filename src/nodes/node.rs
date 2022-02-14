/// Base trait required by nodes of segment trees
pub trait Node {
    /// This type corresponds to the type of the information to create the node with [Node::initialize].
    type Value;
    /// Function to create nodes from saved value, it is assumed that even if there's more data saved in the node, `value` should have enough data to recreate **all** of the data in the node. 
    fn initialize(value: &Self::Value) -> Self;
    /// Function which will combine nodes `a` and `b`, where each corresponds to segments \[i,j\] and \[j+1,k\] respectively, into a node which corresponds to the segment \[i,k\]. This function **must** be associative (taking \* as a symbol for combine, we have that a\*(b\*c)==(a\*b)\*c is true), but need not be commutative (it's not necessarily true that a\*b==b\*a). 
    fn combine(a: &Self, b: &Self) -> Self;
    /// Method which returns a reference to the current saved value.
    fn value(&self) -> &Self::Value;
}
