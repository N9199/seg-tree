# Wide Segment Tree
## Indices
### Recursive Segment Tree Case
`[0] [1,...,B] [B+1,...,2*B],...,[B*B+1,...,B*B+B],...,`

The mapping should be something like:

- `0->[1,...,B]`
- `1->[B+1,...,2*B]`
- `2->[2*B+1,...,3*B]`
- `B->[B*B+1,...,(B+1)*B]`
- `B+1->[(B+1)*B+1,...,(B+2)*B]`
- `k->[k*B+1,...,(k+1)*B]`

So we have that `1+B+B^2+B^3+B^4+...+B^n`, where `n =` height of the tree `= floor(log_B(N))+1`, where `N` is the size of the original `Vec`.
