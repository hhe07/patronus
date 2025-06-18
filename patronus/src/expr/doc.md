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
  - ques: what does the lifetime indicator do?
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

# foreach.rs

## ForEachChild
trait applies to a generic type which also implements the clone trait

trait methods:
- for_each_child: applies a visitor function which can mutate its input
- collect_children: accepts a vector and pushes a clone of each child to the vector
  - ques: how do rust memory semantics work with the c: is it 'returned' to the Expr?
- num_children: presumably return number of children

## impl
- for each Expr, apply the visitor function to its children
- num_children returns the number of child args

crit: solveable with polymorphism (i.e. a separate type for unary, binary, etc. ops?)

# meta.rs
## ExprMap
trait implementors must also implement Debug, Clone, Index<T>, IndexMut<T>
where T is a generic which implements Default, Clone, PartialEq

methods:
- iter: returns an iterator over the ExprRefs and the output generic type
  - ques: is the lifetime because what is iterated over must live as long as the map itself; or vice versa?
- non_default_value_keys: return an iterator over ExprRefs
  - ques: is the lack of lifetime because ExprRef can duplicate?

- get_fixed_point
  - given a key and expression map, if the key is in the map, return the key
  - otherwise, pointer chase for the expression
    - ques: don't understand what's going on there
  - then, change values which led to the fixed point to map to the final value
  - return the final value

## SparseExprMap
fields:
- inner: hash map between an ExprRef and a generic type
- default: a generic type

impl Index:
- attempt to get the value associated with the ExprRef from the inner store, else return a default
- ques: why not return an option?

impl IndexMut:
- attempt to get the value associated with the ExprRef from the inner store, or insert and return a default

impl ExprMap:
- iter
  - takes the inner iterator and applies a transform to return ExprRef values, not references
- non_default_value_keys:
  - takes the inner iterator, filters out default values of value
    - ques: why is the double dereference necessary?

- ques: why does the Self::Output carry over?


## DenseExprMap
- does much of the same as above, but with an underlying vector
  - the ExprMetaDataIter type is necessary to smooth over the fact that ExprRefs do not directly correspond to vector indices?
    - ques: is this correct?

## ExprMetaDataIter
- wraps iterator of vector within DenseExprMetaData with independent index

## ExprSet
trait methods:
- contains: whether or not the implementing type contains an ExprRef
- insert: insert an ExprRef, returning whether the value was present before
- remove: remove an exprref, returning whether the value was present before

- ques: why does insert expect a value, and remove expect a pointer to the value? simply due to hashmap?

## DenseExprSet:
- implements an ExprSet based on a vector of u64 bitfields
- index_to_word_and_bit flattens an ExprRef down to a location in the vector and bit in a bitfield
- crit: could possibly be easier for compiler if use (1u64 << bit) to compare and check if result is nonzero

## SparseExprSet:
- implements an ExprSet based on a hashmap of ExprRefs

# nodes.rs
- WidthInt: baa:WidthInt: maximum width of a bitvector

## BVLitValue
- wraps a BitVecValueIndex to a bit vector value, deriving some helpful traits along the way

methods:
- new: returns a Self wrapped in the type
- get: from a context, get the bv_value associated with own index
  - ques: lifetime required because dealing with references to within context?
- width: return width of own element
- is_true: return whether the width is one and own value is one
- is_false: return whether the width is one and own value is zero
  - crit: possibly condense to one function?

## Expr
an enum defining the possible types of bit-vector expressions and their fields
- WidthInt for easier implementation within simulator

- crit: uneven specification based on alignment -- possible way out? (i.e. place Ite to end of enum?)

impl:
- symbol:
  - given a Type, create a symbol from it
- is_symbol: determine whether an Expr is one of the two types of symbol
- is_bv_lit: determines whether an Expr is a BVLiteral
- get_symbol_name_ref: if the expression is a symbol, return a StringRef corresponding to its name; else return none
- get_symbol_name: call get_symbol_name_ref. if it's a valid reference, return its name from the context; else return none
- is_true, is_false: if the expr is a BVLiteral, return whether it's true or false; else return false
  - crit: better to use option?


## Type
- BVType wraps WidthInt
- ArrayType wraps two WidthInts, one for index width, one for data

a Type is either a BV directly wrapping WidthInt or an ArrayType

impl:
- is_bit_vector, is_array: matches depending on Type
- is_bool: if the Type is a BV, check if the width is one
- get_bit_vector_width, get_array_data_width, get_array_index_width; accessors depending on

- crit: use a union here?

# parse.rs
- parse_expr:
  - constructs a new parser, and parses all of the input
  - performs a type_check on the expr given the context, and returns the reference to the expression if the type check succeeds

## Parser:
fields:
- ctx: a reference to a mutable Context: must live as long as the containing parser
- inp: a reference to the string input
- symbols: a hashmap presumably matching strings to exprreferences




TODO:
- test whether the promised deduplication functionality of the Expr library works
- see where the expr crate lives within the larger project
- crit: move the text representation components to its own crate, have the ast live on its own
- concretisation?
  - presumably this is what traversal.rs does, but there are possibly more efficient approaches
- feature: allow selection of specific parts of the ast, allow certain symbolic inputs within expressions (lazy evaluation of sorts)
  - first, simplify model with inputs as abstract, then 'reattach' inputs, and re-simplify with reasoning about possible inputs

