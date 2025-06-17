# references
- tuple type from NonZeroU32
  - why non-zero?
- ``from_index`` and ``index`` result in a value starting from zero

# ``Context``
fields:
- strings: a hash set with deterministic traversal order
- exprs: a hash set with deterministic traversal order
- values: ``baa::ValueInterner``
- true_expr_ref: ExprRef (cached special value of reference)
- false_expr_ref: ExprRef (cached special value of reference)

constructor:
fills in items with defaults
- what is ``out.zero(1)`` and ``.one(1)``?

methods:
- get_symbol_name:
  - gets the relevant symbol name for an expression in ``exprs``
- add_expr:
  - wraps ``insert_full`` on self.exprs, returns the index as an ExprRef
- string:
  - if the value is in the string store already, return the reference to its index
  - otherwise, add it to the store and add its index
- get_bv_value:
  - what does the lifetime indicator do?
  - gets the reference associated with an index in ``self.values.words()``
- bv_symbol:
  - creates a new BVSymbol with supplied parameters, adds it to the ``exprs`` list
- array_symbol: similar as above, but with an array symbol
- symbol: similar to above, but with an Expr::symbol
- bv_lit: similar to above, but with a BVLiteral
  - ques: confused by purpose of this function
- lit:
  - accepts a value. if the value is a bitvector, add a bitvector literal
  - if the value is an array,
    - add a default SparseArrayVector's bvliteral
    - also add an arrayconstant based on the default's index width and data width
    - then, use the fold to add all subsequent indices and data
    - return the ExprRef associated with the last stored data element
    - crit: is the fold necessary?
    - ques: is the behaviour understanding correct?
- bit_vec_val:
  - attempt to convert ``value`` and the width into a u128 and WidthInt respectively
    - crit: is the width necessary? can it be calculated?
- zero, one:
  - add a bitvector literal with the value in the function name
- zero_array, ones:
  - add a bitvector literal of all zeroes/ones with the width supplied
  - crit: different naming
- get_true, get_false:
  - get the true and false expressions
  - crit: necessary?
- equal
  - check inputs, then add either a bitvector or array equality expression
- ite, implies, greater_signed, greater, greater_or_equal_signed, greater_or_equal: similar to above
- not, negate, and, or, xor, shift_left, arithmetic_shift_right, shift_right: similar to above
- add, sub, mul, div, signed_div, signed_mod, signed_remainder, remainder, concat: similar to above
- slice, zero_extend, sign_extend, extend: similar to above
- array_store: add an arraystore expression based on array, index, data
- array_const, array_read: similar to above
- builder:
  - applies a function to a reference version of a context (Builder)
  - the builder: uses explicit borrows to modify the referenced context
  - ques: what is the borrow checker issue?
  - crit: ``ContextBuilder`` to make it more explicit?

notably, many methods work on ExprRefs

Index<ExprRef, StringRef>:
- allows indexing by an ExprRef into exprs/strings respectively


# eval.rs

## SymbolValueStore
fields:
- arrays: a vector of baa arrayvalues
- bit_vec_words: a vector of words
- lookup: a hashmap of ``ExprRef`` (from context.rs) to a storeindex

- crit: why not directly have the hashmap go from exprref to either an arrayvalue or a word?

methods:
- define_bv
  - assert if the lookup table contains the symbol already
  - otherwise, insert the symbol into the lookup table with the start at the end of the symbolic store, and push the words into bit_vec_words
  - ques: confirming, are lifetimes required because the function needs to live as long as the reference?
  - ques: are any issues caused with the variable word length? the code here (arbitrarily) feels jank
- update_bv
  - retrieves the index form the lookup table, and then assigns the relevant words to a value
  - crit: new index thing feels unintuitive: assume that assign handles length correctly? if it's true, this isn't very clear
- define_array
  - assert if the lookup table contains the symbol already
  - otherwise, insert the item into the arrays vector
- update_array
  - update the arrays vector with a given value
- update:
  - wrap above updators, depending on which type of value it is
- clear
  - clear all fields

impl GetExprValue:
- get_bv
  - return the bitvector word associated with a symbol
  - crit: clunk of the context, and needing to exprref.get_bv_type(ctx)?
- get_array:
  - return the array value associated with a symbol
- crit: why the clones?

impl From:
- populate symbolic value store based on various combinations of reference and value
- crit: getexpr stuff: why so many nones?
  - also, does the number of different types of combinations say something negative?


un_op / bin_op:
- based on a stack of bitvector values, pop either one or two values from the stack, operate on them, and push the result
- crit: possible to make generic?

eval_bv_expr, eval_array_expr, eval_expr:
- all wrap around eval_expr_internal, and essentially return the result of the correct type

eval_expr_internal
- for the given expression:
  - if there aren't args available,
    - attempt to obtain them from the value 'store' (of type GetExprValue)
    - otherwise, push children
    - crit: possible path through this code which breaks it? i.e. no child, no args available, and no arguments on stack
  - otherwise, match based on expression and evaluate the expression
    - todo: some todos here
