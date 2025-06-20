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

### Arg
enum:
- E: ExprRef
- C: WidthInt

### ArgTpe
enum:
E,C

- ques: unsure what this does

impl Parser:
- new:
  - trims input of leading / trailing spaces, attaches context, creates new symbol table
- parse_expr_all:
  - attempts to call parse_expr, returns ExprRef.
  - if empty string, assert error
  - crit: assertion before call?
- parse_expr:
  - try parsing using the function, bv_lit, or symbol methods. panic if parsing fails
  - if the result is additionally a slice, attempt to create a slice from the correct type.
- width_int:
  - for a given match, attempt to parse it into a WidthInt?
- try_parse_fun:
  - attempt to match against a function and create it
- try_pars_bv_lit
  - attempt to match against the different types of bitvector literal (bin, dec, hex, or bool) and add it
  - crit: inconsistent naming
- try_parse_symbol:
  - attempt to create a symbol out of the line, optionally with a width
  - ques: "do we have an explicit bv type" comment
  - crit: potentially simplifiable?
- make_fun:
  - add a symbol to context based on the function id
  - crit: again, hardcoding -- more efficient way leveraging type system?
  - TODO here
- parse_args
  - given the type of function and a table of expected args, attempt to create a vector containing the args to current input in order
- try_parse_width_int
  - attempt to parse a line for a WidthInt (a decimal number)
  - ques: any way this could break?
- consume_r
  - consume from the first match of an arbitrary regex
- consume_m
  - consume (advance in input string) from the end of a match to the end of the input
- consume_c
  - given a list of captures, consume from after the first
- crit: possible parallelism in dividing into lines first?
- crit: have regex in a file and not hard-coded?

const FUNCTIONS: a list of functions
const FUNCTION_ARGS: a list of args for said functions

lazy_static! block: regex expressions for the possible parse inputs
  - interesting regex set thing: used when constructing a function id (a direct map to an index in array)

- ques: how to accomodate comments? more generic optional extra arg?

# serialize.rs
## SerializableIrNode
trait methods:
- serialize: given a context and writer, return a result
- serialize_to_str: use the serialize function to write result into a vec (presumably utf8), and return its contents as a string

impl for Expr:
- wraps the serialize_expr function, using a lambda which returns true to enable max descent

- serialize_expr:
  - parameter serialize_child must be a function of ExprRef and W that returns a bool as a result
  - W must implement the Write trait
  - matching over Expr, write a suitable representation of the expr to the result
    - crit: generic form for each type of formula?
  - symbol names are obtained from ctx
  - serialize_child is used to continue recursion
- serialize_expr_ref:
  - obtains the expr corresponding to ExprRef from context, and continues serialisation
- SerializableIRNode for ExprRef:
  - presumably to handle some cases where descent is limited.
  - serializes based on the entry in the context
- SerializeIRNode for Type:
  - write self to writer


# simplify.rs
- simplify_single_expression:
  - given a context and an ExprRef, creates a new simplifier and applies it to only that ExprRef

## Simplifier:
fields:
- cache: a generic ExprMap over an option of ExprRefs

impl Simplifier:
- new: creates cache
- simplify:
  - given context and an ExprRef, run do_transform_expr with ctx, FixedPoint, the cache, a vector containing the ExprRef, and simplify as parameters
  - attempt to get the fixed point expression from cache and return it as result

- simplify:
  - given a context, an exprref, and a list of child exprrefs:
    - matching against a copy of the Expr and its children, call the simplifier for the Expr type
      - crit: in-place method?
- simplify_ite:
  - crit: tru, fals are clunky names?
  - if both returns are equal, return one
  - if the condition is constant, return the value it corresponds to
  - if the returns are 'opposing' bools, return the corresponding logical function
  - if one of the returns is a bool, return a logical function
    - crit: way of deduplicating?

## Lits
enum:
- Two: two literals
- One: a tuple containing a tuple of literal and ExprRef, and then an ExprRef

- find_lits_commutative:
  - given two children of a commutative function, return the literals
    - if both children are literals, return a pair
    - if only one is a literal, return a One with the inner tuple containing the literal
      - crit: why additionally return both?
- find_one_concat:
  - given two children of a function, if either is a concat, return the concat components plus the non-concat child
    - crit: width discarded?
- simplify_bv_equal:
  - if args equal (key => value), return True
  - if two literals, make sure actually different values (debug). since diff. hash implies diff. value, return False
  - if reduces to a comparison (i.e. x == True/False), return equivalent expression
    - ques: any problems with >1 width?
  - if comparing against a concat, compare the parts individually against their corresponding parts of ``other``
    - crit: is this actually more efficient?
    - crit: simplification of 'split' concats: i.e. {x, 2'b00} and a signal to return a constant or something
- simplify_bv_and
  - if the values are equivalent, return input
  - if both of the values are literals, return another literal with the concrete value
  - if one of the values is a literal:
    - if one is zero, return zero
    - if one is one, return the other value
    - if concat & mask, split mask through concat
      - crit: further descent?
    - otherwise, if a normal signal and a mask, reduce into a concat of active wires and the zeroes
      - crit: what if turned into a concat of ternaries? also, performance of code block and within solver?
    - otherwise, if neither value is a literal:
      - if either is a not of the other, return zero
      - if de morgan's, apply the relevant simplification
- simplify_bv_or
  - if args equal, return the arg
  - if both values literal, get concrete value
  - if one value literal:
    - if one is zero, return arg
    - if one is one, return 1
    - todo: concat | mask type of deal
  - otherwise, handle a | !a and de morgan's law cases
    - crit: de morgan's law is a bit bad?
- simplify_bv_xor
  - xor with self is zero
  - if both values literal, get concrete value
  - if one value literal:
    - if one is zero, return arg
    - if one is 1, return !a
    - todo: concat xor mask type of deal
  - xor with inverse of self is one
- simplify_bv_not
  - undo double negations
  - replace negations of literals with literal
- simplify_bv_zero_ext
  - eliminate extension by zero bits
  - replace extension of literal with literal
  - otherwise, normalise to concat
- simplify_bv_sign_ext
  - eliminate extension by zero bits
  - replace extension of literal with literal
  - otherwise, don't simplify?
    - todo: reduce to concat?
- simplify_bv_concat:
  - if the concat contains another concat, shift it over to be right recursive
  - if the components are literals, replace to literal
  - if one is literal and the other is a concat:
    - if it's possible to 'extend' the literal, do so
  - if both are adjacent slices of the same bitvector, return a slice of the whole
- simplify_bv_slice
  - if slicing a slice, combine slices
  - if slicing a constant, extract a constant
  - if slicing a concat, depending on the slice point, either return a slice of only one signal or a more restricted concat
  - if slicing a sign extend, either slice the internal (reducing size), or expand to size
  - if slicing an ite, return an ite whose arguments are sliced
  - if slicing a not, reverse order (not the slice)
    - ques: will this possibly result in inefficiency: what if the not of the whole symbol is reused
    - ques: general guidelines / restrictions on whether to keep signals unsliced or sliced
  - if slicing an xor or or, do the same
  - if slicing an arithmetic op with carry, don't actually do the carry
- simplify_bv_shift_left:
  - replace operation on literals with another literal
  - if shift is by a literal:
    - if exceeds width, return zeroes.
    - if no shift, return the same value
    - otherwise, replace with a concat
  - if shift amount is larger than a u64, clear to zero
    - crit: will this cause problems with larger system (i.e. does it correspond to word size?)
- simplify_bv_shift_right:
  - replace operation on literals with another literal
  - if shift is by a literal:
    - if exceeds width, return zeroes
    - if no shift, return same value
    - otherwise, replace with a zero extension
- simplify_bv_arithmetic_shift_right
  - similar as above, except replace with sign extension
- simplify_bv_add:
  - replace operation on literals with another literal
  - replace addition of zero with original value
- simplify_bv_mul:
  - replace operation on literals with another literal
  - multiplication by zero or 1 gets replaced with a constant
  - multiplication by power of two gets replaced with a shiftleft
- TODO: simplify_bv_sub?

- crit: don't love the presence of arithmetic ops? but also idk
- crit: zero extension seems redundant with concat


# transform.rs
## ExprTransformMode
derives Debug, Copy, Clone, Eq, PartialEq
types:
- SingleStep
- FixedPoint

simple_transform_expr:
- "transform with a single step (no fixed point) and no persistent cache"
- expects a context, an expression reference, and a transformation function
  - the transformation function should accept a context, exprref, and a list of exprrefs; and output an option exprref
- create a SparseExprMap
- call do_transform_expr
- get the mapped value corresponding to the input exprref from the cache

do_transform_expr:
- create a vector for children
- while there are still exprrefs on the todo stack:
  - reset children vector, children_changed = false, all_transformed = true
  - for each child of an expr_ref:
    - transformed_child is either the fixed point version, or the one directly from the store of transformed values
    - match:
      - if something was returned:
        - if it's a distinct reference, something's changed: set children_changed
        - add the child expression to children vector
      - if nothing was returned:
        - if everything transformed, push the expression
        - reset all_transformed and push the child
  - if not all children were transformed, restart loop iteration (child now on top)
  - then, call the transform
    - if the result was something, set new_expr_ref to it
    - if the result was nothing:
      - if children changed, update children
      - if no children changed and transform does not apply changes, keep old expression
  - store transformed expression (possibly none)
  - if in fixed point mode, and not at the fixed point and if the transformed result is not in the transformed store
      - ques: (expr_ref == new_expr_ref)??
      - add the new expression back to todo

update_expr_children:
- match the expr type
  - replace inputs to the expression with new, possibly changed children
  - TODO: some expressions could be missing
- add the new expression to the context

# traversal.rs
bottom_up:
- operates on generic R, returns R
- accepts a context, ExprRef, and a function f with mutable receiver that accepts a context, exprref, and a reference to a slice of R
- call bottom_up_multi_pat, with a get_children function which adds each child of the expression to a vector

bottom_up_multi_pat
- accepts a context, ExprRef, a get_children function, and a function f like above
- create a todo vector, a stack, and a child vector
  - todo vector stores tuples of (exprref, bool)
  - the bool represents "bottom up"
- while the todo list is not empty:
  - obtain a reference to the expr pointed at by the stack entry
  - if not bottom up:
    - assert child_vec is empty
    - get children into child_vec
    - push the first child onto todo as bottom up; push others as not bottom up
    - continue
  - exiting this implies that have arrived at 'the bottom layer'
  - get the child values from stack
    - len returns num elements, not the capacity
    - in base case, there are no values
  - call f with the context, exprref, and values
  - truncate the stack to the values not used in above
  - push the result to stack
- ultimately, there should be one thing on stack, which is popped and unwrapped

- ques: what's the 'match patterns with multiple nodes' thing -- the fact that stack and such can be of semi-arbitrary len?

## enum TraversalCmd
- values Stop, Continue

top_down:
- accepts a context, ExprRef, and a function f with mutable receiver that accepts a context and exprref and reports whether to stop or continue
- creates a todo vector
- while there are still exprs to visit:
  - check whether to continue
  - if should continue, add each child of the current expr to todo

- ques: can f in top_down mutate, for example, a list within its context to do something during the expression visiting?



TODO:
- test whether the promised deduplication functionality of the Expr library works
- see where the expr crate lives within the larger project
- crit: move the text representation components to its own crate, have the ast live on its own
- concretisation?
  - presumably this is what traversal.rs does, but there are possibly more efficient approaches
- feature: allow selection of specific parts of the ast, allow certain symbolic inputs within expressions (lazy evaluation of sorts)
  - first, simplify model with inputs as abstract, then 'reattach' inputs, and re-simplify with reasoning about possible inputs

- don't understand fixed point thing


- context compression / elimination of unnecessary terms?

- how to handle de morgan's and other shit in simplfication
- k-maps?

reducing all to ternaries?

commutative trait?

automatic simplification while constructing from signal source: will this create more inefficiency?

do enum structs all have to have different type?
