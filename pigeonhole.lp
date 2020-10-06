#const h = 6.
#const p = h+1.

1 { place(P,H) : H = 1..h } 1 :- P = 1..p.
{ place(P,H) : P = 1..p } 1 :- H = 1..h.