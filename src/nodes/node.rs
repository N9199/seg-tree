/// Base trait required by nodes of segment trees. A type which implements this trait is essentially a "compressed" representation of a non-empty segment of [values](Node::Value). 
pub trait Node {
    /// This type corresponds to the type of the information to create the node with [Node::initialize].
    type Value;
    /// Function to create nodes from saved value, it is assumed that even if there's more data saved in the node, `value` should have enough data to create **all** of the data of a node of a segment segment of exactly one element.
    fn initialize(value: &Self::Value) -> Self;
    /// Function which will combine nodes `a` and `b`, where each corresponds to segments `[i,j]` and `[j+1,k]` respectively, into a node which corresponds to the segment `[i,k]`. This function **must** be associative (taking \* as a symbol for combine, we have that a\*(b\*c)==(a\*b)\*c is true), but need not be commutative (it's not necessarily true that a\*b==b\*a).
    fn combine(a: &Self, b: &Self) -> Self;
    /// Method which returns a reference to the current saved value.
    fn value(&self) -> &Self::Value;
}
