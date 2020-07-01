// This isn't actually Rust, it's actually some nonsense language I'm working on.
// I've just set .rs as the extension so that GitHub will use Rust's syntax highlighting.

extern puts(ptr: i64);
extern printf(ptr: i64, val: i64);
extern exit(ret_code: i64);
extern malloc(size: i64) -> i64;
extern realloc(ptr: i64, size: i64) -> i64;
extern free(ptr: i64);
extern memcpy(dest: i64, src: i64, n: i64) -> i64;
extern memmove(dest: i64, src: i64, n: i64) -> i64;
extern signal(signum: i64, function_pointer: i64) -> i64;
extern write(fd: i64, buf: i64, count: i64) -> i64;

fn _builtin_match_failure() {
    panic("Every match case failed; memory corruption likely");
}

fn _builtin_sigsegv_handler() {
    panic("Segmentation fault occurred");
}

fn _builtin_sigint_handler() {
    panic("Keyboard interrupt");
}

fn _builtin_sigquit_handler() {
    panic("Quit received");
}

fn _init_signal_handlers() {
    func_ptr: i64;
    SIGINT  := 2;
    SIGQUIT := 3;
    SIGSEGV := 11;
    -| mov qword [${local_func_ptr}], func__builtin_sigint_handlerpP
    signal(SIGINT, func_ptr);
    -| mov qword [${local_func_ptr}], func__builtin_sigquit_handlerpP
    signal(SIGQUIT, func_ptr);
    -| mov qword [${local_func_ptr}], func__builtin_sigsegv_handlerpP
    signal(SIGSEGV, func_ptr);
}

_init_signal_handlers();

fn panic(msg: str) {
    puts("===== Panic =====".chars.raw);
    traceback_stack_ptr: UnsafePtr<i64>;
    -| mov qword [${local_traceback_stack_ptr}], TracebackStack
    traceback_stack_end: UnsafePtr<i64>;
    -| mov rax, [TracebackStackPointer]
    -| mov [${local_traceback_stack_end}], rax
    while traceback_stack_ptr.raw != traceback_stack_end.raw {
        traceback_stack_ptr += 1;
        puts(traceback_stack_ptr~);
    }
    puts(msg.chars.raw);
    exit(1);
}

fn assert(condition: bool, msg: str) {
    if not condition {
        panic(msg);
    }
}

struct unit {}

struct bool: 1 {
    static fn _make_builtin_bools(x: i64) -> bool {
        -| mov rax, [${arg_x}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], al
    }

    static fn operator_not(b: bool) -> bool {
        if b {
            return false;
        }
        return true;
    }

    static fn operator_and(a: bool, b: bool) -> bool {
        if a {
            if b {
                return true;
            }
        }
        return false;
    }

    static fn operator_or(a: bool, b: bool) -> bool {
        if a {
            return true;
        }
        if b {
            return true;
        }
        return false;
    }

    static fn to_str(a: bool) -> str {
        if a {
            return "true";
        }
        return "false";
    }
}

false := bool::_make_builtin_bools(0);
true  := bool::_make_builtin_bools(1);

struct u8: 1 {
    fn new() {
        -| mov rax, [${arg_self}]
        -| mov byte [rax], 0
    }

    static fn operator_eq(x: u8, y: u8) -> bool {
        -| mov ecx, 1
        -| mov al, [${arg_x}]
        -| cmp al, [${arg_y}]
        -| jz ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_neq(x: u8, y: u8) -> bool {
        -| mov ecx, 1
        -| mov al, [${arg_x}]
        -| cmp al, [${arg_y}]
        -| jnz ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn from_char(s: &str) -> u8 {
        assert(s.length == 1, "from_char argument must be of length 1");
        return s.chars~;
    }

    fn to_str() -> str {
        return str::from_pointer(1, UnsafePtr::<u8>::get_pointer_to(self));
    }
}

struct i64: 8 {
    fn new() {
        // Zero initialize the integer by default.
        -| mov rax, [${arg_self}]
        -| mov qword [rax], 0
    }

    static fn operator_add(x: i64, y: i64) -> i64 {
        -| mov rax, [${arg_x}]
        -| add rax, [${arg_y}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn operator_ip_add(x: &i64, y: i64) {
        -| mov rax, [${arg_x}]
        -| mov rbx, [${arg_y}]
        -| add [rax], rbx
    }

    static fn operator_sub(x: i64, y: i64) -> i64 {
        -| mov rax, [${arg_x}]
        -| sub rax, [${arg_y}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn operator_ip_sub(x: &i64, y: i64) {
        -| mov rax, [${arg_x}]
        -| mov rbx, [${arg_y}]
        -| sub [rax], rbx
    }

    static fn operator_neg(x: i64) -> i64 {
        -| xor rax, rax
        -| sub rax, [${arg_x}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn operator_mul(x: i64, y: i64) -> i64 {
        -| mov rax, [${arg_x}]
        -| mul qword [${arg_y}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn operator_ip_mul(x: &i64, y: i64) {
        x = x * y;
    }

    static fn operator_eq(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jz ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_neq(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jnz ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_lt(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jl ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_le(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jle ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_gt(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jg ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    static fn operator_ge(x: i64, y: i64) -> bool {
        -| mov ecx, 1
        -| mov rax, [${arg_x}]
        -| cmp rax, [${arg_y}]
        -| jge ${prefix}_eq
        -| mov ecx, 0
        -| ${prefix}_eq:
        -| mov rbx, [${arg_@return_pointer}]
        -| mov byte [rbx], cl
    }

    fn to_str() -> str {
        sb: StringBuilder;
        val := self;
        if val < 0 {
            val = -val;
            sb += "-";
        }
        return sb.to_str();
    }
}

fn max(x: i64, y: i64) -> i64 {
    if x < y {
        return y;
    }
    return x;
}

struct UnsafePtr<T> {
    raw: i64;

    static fn get_pointer_to(obj: &T) -> UnsafePtr<T> {
        -| mov rax, [${arg_obj}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn operator_deref(self: UnsafePtr<T>) -> &T {
        -| mov rax, [${arg_self}]
        -| mov rbx, [${arg_@return_pointer}]
        -| mov [rbx], rax
    }

    static fn assign(self: UnsafePtr<T>, value: &T) {
        len := TypeInfo::<T>::sizeof;
        -| mov rdi, [${arg_self}]
        -| mov rsi, [${arg_value}]
        -| mov rcx, [${local_len}]
        -| rep movsb
    }

    fn shift_by_bytes(byte_count: i64) -> UnsafePtr<T> {
        return UnsafePtr::<T>{self.raw + byte_count};
    }

    fn operator_add(index: i64) -> UnsafePtr<T> {
        return UnsafePtr::<T>{
            self.raw + index * TypeInfo::<T>::sizeof
        };
    }

    fn operator_ip_add(index: i64) {
        self = self + index;
    }

    fn operator_radd(index: i64) -> UnsafePtr<T> {
        return self + index;
    }

    fn operator_sub(index: i64) -> UnsafePtr<T> {
        return self + -index;
    }

    fn operator_ip_sub(index: i64) {
        self = self - index;
    }

    fn operator_getindex(index: i64) -> &T {
        return (self + index)~;
    }
}

struct PtrCast<T, U> {
    static fn cast(ptr: UnsafePtr<T>) -> UnsafePtr<U> {
        return UnsafePtr::<U>{ptr.raw};
    }
}

struct Pair<A, B> {
    first: A;
    second: B;
}

struct Mem<T> {
    raw: i64;

    static fn alloc(size: i64) -> Mem<T> {
        ref_count_size := TypeInfo::<i64>::sizeof;
        return Mem::<T>{malloc(ref_count_size + size) + ref_count_size};
    }

    fn copied() {
        self._ref_count() += 1;
    }

    fn drop() {
        self._ref_count() -= 1;
        if self._ref_count() == 0 {
            T::drop_raw_ptr(self.raw);
            free(self.raw);
        }
    }

    fn _ref_count() -> &i64 {
        return (UnsafePtr::<i64>{self.raw} - 1)~;
    }
}

struct MemUnsafe {
    static fn drop_raw_ptr(raw: i64) {}
}

struct Box<T> {
    mem: Mem<Box<T>>;

    optional skipdefault fn new() {
        self.mem = Mem::<Box<T>>::alloc(TypeInfo::<T>::sizeof);
        self~.new();
    }

    optional fn clone() -> Box<T> {
        new_box: Box<T>;
        new_box~ = self~.clone();
        return new_box;
    }

    fn operator_deref() -> &T {
        return UnsafePtr::<T>{self.mem.raw}~;
    }

    static fn drop_raw_ptr(raw: i64) {
        UnsafePtr::<T>{raw}~.drop();
    }
}

// Ugh, this function is terrible, and should be replaced by proper casting later.
fn make_u8(x: i64) -> u8 {
    result := u8{};
    -| mov rax, [${arg_x}]
    -| mov [${local_result}], al
    return result;
}

// This constant is used for making slices.
// FIXME: There's currently a bug where integer literals larger
// than fit in an x86 immediate field aren't handled correctly.
// This works around that.
END := 1;
for _ in 0..16 {
    END *= 10;
}

struct Slice {
    start: i64;
    stop: i64;
    type IterT = Interval<i64>::IterT;

    static fn from(interv: Interval<i64>) -> Slice {
        return Slice{interv.start, interv.stop};
    }

    fn fixup_for_length(length: i64) {
        if self.start < 0 {
            self.start += length;
        }
        if self.start < 0 {
            self.start = 0;
        }
        if self.stop < 0 {
            self.stop += length;
        }
        if self.stop > length {
            self.stop = length;
        }
        if self.stop < self.start {
            self.stop = self.start;
        }
    }

    fn to_interval() -> Interval<i64> {
        return Interval::<i64>{self.start, max(self.start, self.stop)};
    }

    fn to_iter() -> IntervalIterator<i64> {
        return self.to_interval().to_iter();
    }
}

struct StrIterator {
    index: i64;
    base_string: str;
    type ValT = str;

    fn next() -> Maybe<str> {
        if self.index < self.base_string.len() {
            val := Maybe::<str>::Some{self.base_string[self.index]};
            self.index += 1;
            return val;
        }
        return Maybe::<str>::None{};
    }
}

struct P<T> {
    static fn any(x: &T) -> bool {
        for b in x {
            if b {
                return true;
            }
        }
        return false;
    }

    static fn all(x: &T) -> bool {
        for b in x {
            if not b {
                return false;
            }
        }
        return true;
    }
}

struct str {
    length: i64;
    chars: UnsafePtr<u8>;
    allocation: Mem<MemUnsafe>;
    type IterT = StrIterator;

    skipdefault fn new() {
        self = "";
    }

    static fn from_pointer(length: i64, ptr: UnsafePtr<u8>) -> str {
        new_allocation := Mem::<MemUnsafe>::alloc(length + 1);
        new_chars := UnsafePtr::<u8>{new_allocation.raw};
        memcpy(new_allocation.raw, ptr.raw, length);
        // Add null termination.
        new_chars[length] = make_u8(0);
        return str{
            length,
            new_chars,
            new_allocation,
        };
    }

    static fn from_cstr(ptr: UnsafePtr<u8>) -> str {
        length := 0;
        while ptr[length] != make_u8(0) {
            length += 1;
        }
        return str::from_pointer(length, ptr);
    }

    fn len() -> i64 {
        return self.length;
    }

    fn write_and_advance(dest: &UnsafePtr<u8>) {
        memcpy(dest.raw, self.chars.raw, self.length);
        dest += self.length;
    }

    fn operator_add(other: str) -> str {
        // The +1 is for a null termination, for convenience.
        new_allocation := Mem::<MemUnsafe>::alloc(self.length + other.length + 1);
        dump := UnsafePtr::<u8>{new_allocation.raw};
        self.write_and_advance(dump);
        other.write_and_advance(dump);
        "\0".write_and_advance(dump);
        return str{
            self.length + other.length,
            UnsafePtr::<u8>{new_allocation.raw},
            new_allocation,
        };
    }

    fn operator_ip_add(other: str) {
        self = self + other;
    }

    fn operator_getindex(index: i64) -> str {
        return self[Slice{index, index + 1}];
    }

    fn operator_getindex(slice: Slice) -> str {
        slice.fixup_for_length(self.length);
        return str{
            slice.stop - slice.start,
            self.chars + slice.start,
            self.allocation,
        };
    }

    // Use this to eliminate the reference into the parent.
    fn detach() -> str {
        return str::from_pointer(self.length, self.chars);
    }

    fn to_iter() -> StrIterator {
        return StrIterator{0, self};
    }

    fn to_str() -> str {
        return self;
    }

    fn operator_eq(other: str) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for i in 0..self.len() {
            if self.chars[i] != other.chars[i] {
                return false;
            }
        }
        return true;
    }

    fn operator_neq(other: str) -> bool {
        return not (self == other);
    }

    fn contains(other: str) -> bool {
        other_len := other.len();
        for start_pos in 0..(self.len() - other_len + 1) {
            if self[start_pos..(start_pos + other_len)] == other {
                return true;
            }
        }
        return false;
    }
}

FD_STDIN  := 0;
FD_STDOUT := 1;
FD_STDERR := 2;

fn print_str(s: &str) {
    write(FD_STDOUT, s.chars.raw, s.length);
}

fn print(s: &str) {
    print_str(s);
    print_str("\n");
}

// TODO: This function should just be polymorphic, once I implement forall.
fn print(n: i64) {
    show_number(n);
}

// This is a special magical type that has an array length baked in.
struct _BuiltinArray<T> {
    // The storage of this type is magically inserted by the compiler.

    fn to_ptr() -> UnsafePtr<T> {
        return PtrCast::<_BuiltinArray<T>, T>::cast(
            UnsafePtr::<_BuiltinArray<T>>::get_pointer_to(self)
        );
    }

    fn operator_getindex(index: i64) -> &T {
        //assert(0 <= index, "Array index underflow");
        //assert(index < _MAGIC_ARRAY_LENGTH, "Array index underflow");
        return self.to_ptr()[index];
    }
}

struct VecKishkes<T> {
    length: i64;
    capacity: i64;
    contents: UnsafePtr<T>;

    fn drop() {
        self.empty();
        free(self.contents.raw);
    }

    fn clone() -> VecKishkes<T> {
        new_contents: UnsafePtr<T>;
        if self.capacity > 0 {
            new_contents = UnsafePtr::<T>{malloc(
                TypeInfo::<T>::sizeof * self.capacity
            )};
            for i in 0..self.length {
                new_contents[i] = self.contents[i];
            }
        }
        return VecKishkes::<T>{
            self.length, self.capacity, new_contents,
        };
    }

    fn _reallocate(new_capacity: i64) {
        if new_capacity < self.length {
            panic("BUG: Attempt to shorten a Vec below its length!");
        }
        if new_capacity <= self.capacity {
            return;
        }
        //printf("Realloc raw pointer: %p\n".chars.raw, self.contents.raw);
        //printf("New size: %llu\n".chars.raw, TypeInfo::<T>::sizeof * new_capacity);
        self.contents = UnsafePtr::<T>{realloc(
            self.contents.raw,
            TypeInfo::<T>::sizeof * new_capacity,
        )};
        self.capacity = new_capacity;
    }

    fn append(val: &T) {
        if self.length == self.capacity {
            // TODO: Instead multiply by 1.5.
            new_capacity := max(self.capacity + 4, self.capacity * 2);
            self._reallocate(new_capacity);
        }
        self.contents[self.length] = val;
        self.length += 1;
    }

    fn pop() -> T {
        if self.length == 0 {
            panic("Pop from empty Vec.");
        }
        self.length -= 1;
        result := self.contents[self.length];
        self.contents[self.length].drop();
        return result;
    }

    fn empty() {
        for i in 0..self.length {
            self.contents[i].drop();
        }
        self.length = 0;
    }
}

struct VecIterator<T> {
    index: i64;
    vec: Vec<T>;
    type ValT = T;

    fn next() -> Maybe<T> {
        if self.index < self.vec.len() {
            val := Maybe::<T>::Some{self.vec[self.index]};
            self.index += 1;
            return val;
        }
        return Maybe::<T>::None{};
    }
}

struct Vec<T> {
    kishkes: Box<VecKishkes<T>>;
    type IterT = VecIterator<T>;

    fn clone() -> Vec<T> {
        result: Vec<T>;
        result.kishkes~ = self.kishkes~.clone();
        return result;
    }

    fn len() -> i64 {
        return self.kishkes~.length;
    }

    fn to_iter() -> VecIterator<T> {
        return VecIterator::<T>{0, self};
    }

    optional fn to_str() -> str {
        sb: StringBuilder;
        sb += "[";
        comma := false;
        for x in self {
            if comma {
                sb += ", ";
            }
            sb += x.to_str();
            comma = true;
        }
        sb += "]";
        return sb.to_str();
    }

    optional fn contains(other: &T) -> bool {
        for x in self {
            if x == other {
                return true;
            }
        }
        return false;
    }

    fn _wrap_index(i: i64) -> i64 {
        length := self.len();
		if i < 0 {
			i += length;
		}
		if i < 0 {
			panic("Vector index underflow.");
		}
		if i >= length {
			panic("Vector index overflow.");
		}
		return i;
	}

    fn operator_getindex(index: i64) -> &T {
        index = self._wrap_index(index);
        return self.kishkes~.contents[index];
    }

    fn operator_getindex(slice: Slice) -> Vec<T> {
        slice.fixup_for_length(self.len());
        result: Vec<T>;
        for i in slice {
            result.append(self[i]);
        }
        return result;
    }

    fn append(val: &T) {
        self.kishkes~.append(val);
    }

    fn pop() -> T {
        return self.kishkes~.pop();
    }

    fn empty() {
        self.kishkes~.empty();
    }
}

// For avoiding n^2 complexity of building up a string.
struct StringBuilder {
    contents: Vec<u8>;

    fn new() {
        self.contents.new();
    }

    fn len() -> i64 {
        return self.contents.len();
    }

    fn operator_ip_add(s: &str) {
        orig_length := self.contents.len();
        new_length := orig_length + s.len();
        self.contents.kishkes~._reallocate(new_length);
        self.contents.kishkes~.length = new_length;
        end_pointer := self.contents.kishkes~.contents + orig_length;
        s.write_and_advance(end_pointer);
    }

    fn empty() {
        self.contents.empty();
    }

    fn to_str() -> str {
        // TODO: One day it would be super cool to make this string share its allocation with the vec.
        // I think this is a little hard, because of the requirement of null termination on a string.
        return str::from_pointer(
            self.contents.len(),
            self.contents.kishkes~.contents,
        );
    }
}

// For now this is a ridiculous implementation that does a linear scan over a Vec.
// TODO: Replace with a proper hash table.
struct Dict<K, V> {
    kishkes: Box<VecKishkes<Pair<K, V>>>;
    type IterT = VecIterator<Pair<K, V>>;

    fn new() {
        self.kishkes.new();
    }

    fn clone() -> Dict<K, V> {
        result: Dict<K, V>;
        result.kishkes~ = self.kishkes~.clone();
        return result;
    }

    fn to_iter() -> VecIterator<Pair<K, V>> {
        return Vec::<Pair<K, V>>{self.kishkes}.to_iter();
    }

    fn _find_cell(k: &K) -> Maybe<UnsafePtr<Pair<K, V>>> {
        kishkes := self.kishkes~;
        for i in 0..kishkes.length {
            if kishkes.contents[i].first == k {
                return Maybe::<UnsafePtr<Pair<K, V>>>::Some{
                    kishkes.contents + i
                };
            }
        }
        return Maybe::<UnsafePtr<Pair<K, V>>>::None{};
    }

    fn insert(k: &K, v: &V) {
        match self._find_cell(k) {
            Some{ptr} -> ptr~.second = v;
            None -> self.kishkes~.append(Pair::<K, V>{k, v});
        };
    }

    fn operator_getindex(k: &K) -> &V {
        match self._find_cell(k) {
            Some{ptr} -> { return ptr~.second; };
            None -> panic("Index not found.");
        };
    }

    fn contains(k: &K) -> bool {
        return match self._find_cell(k) {
            Some -> true;
            None -> false;
        };
    }
}

fn show_number(x: i64) {
    printf("Value: %llu\n".chars.raw, x);
}

struct Maybe<T> {
	variant Some {
		contents: T;
	}
    variant None {}

    fn operator_unwrap() -> T {
        match self {
            Some{x} -> { return x; };
            None -> panic("Maybe unwrap failure");
        };
    }
}

struct Result<T> {
    variant Ok {
        contents: T;
    }
    variant Err {
        message: str;
    }

    fn operator_unwrap() -> T {
        match self {
            Ok{x} -> { return x; };
            Err{msg} -> panic("Result unwrap failure: " + msg);
        };
    }
}

struct Interval<T> {
    start: T;
    stop: T;
    type IterT = IntervalIterator<T>;

    fn to_iter() -> IntervalIterator<T> {
        return IntervalIterator::<T>{self.start, self.stop};
    }
}

struct IntervalIterator<T> {
    current: T;
    stop: T;
    type ValT = T;

    fn next() -> Maybe<T> {
        if self.current == self.stop {
            return Maybe::<T>::None{};
        }
        ret := self.current;
        self.current += 1;
        return Maybe::<T>::Some{ret};
    }
}

struct EnumerateIterator<T> {
    counter: i64;
    iterator: T::IterT;

    fn next() -> Maybe<Pair<i64, T::IterT::ValT>> {
        self.counter += 1;
        r := self.iterator.next();
        return match r {
            Some{x} -> Maybe::<Pair<i64, T::IterT::ValT>>::Some{
                Pair::<i64, T::IterT::ValT>{self.counter - 1, x}
            };
            None -> Maybe::<Pair<i64, T::IterT::ValT>>::None{};
        };
    }
}

struct Enumerate<T> {
    iterator: EnumerateIterator<T>;

    fn to_iter() -> EnumerateIterator<T> {
        return self.iterator;
    }

    static fn over(x: &T) -> Enumerate<T> {
        return Enumerate::<T>{
            EnumerateIterator::<T>{0, x.to_iter()}
        };
    }
}

fn operator_range(start: i64, stop: i64) -> Slice {
    return Slice{start, stop};
}


