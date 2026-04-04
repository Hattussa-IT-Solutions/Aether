use aether::interpreter;
use aether::interpreter::environment::Environment;
use aether::interpreter::values::Value;
use aether::lexer::scanner::Scanner;
use aether::parser::parser::Parser;

fn run_get(source: &str, var: &str) -> Value {
    let mut scanner = Scanner::new(source, "test.ae".to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("parse failed");
    let mut env = Environment::new();
    interpreter::register_builtins(&mut env);
    interpreter::interpret(&program, &mut env).expect("interpret failed");
    env.get(var).unwrap_or(Value::Nil)
}

fn assert_int(val: &Value, expected: i64) {
    match val {
        Value::Int(n) => assert_eq!(*n, expected, "expected {}, got {}", expected, n),
        _ => panic!("expected Int({}), got {:?}", expected, val),
    }
}

fn assert_float(val: &Value, expected: f64) {
    match val {
        Value::Float(f) => assert!((f - expected).abs() < 0.01, "expected {}, got {}", expected, f),
        Value::Int(n) => assert!((*n as f64 - expected).abs() < 0.01),
        _ => panic!("expected Float({}), got {:?}", expected, val),
    }
}

fn assert_str(val: &Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(s, expected, "expected '{}', got '{}'", expected, s),
        _ => panic!("expected String('{}'), got {:?}", expected, val),
    }
}

fn assert_bool(val: &Value, expected: bool) {
    match val {
        Value::Bool(b) => assert_eq!(*b, expected),
        _ => panic!("expected Bool({}), got {:?}", expected, val),
    }
}

// ═══════════════════════════════════════════════════════════════
// Classes — basic (20 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_class_single_field() {
    assert_int(&run_get("class C { val: Int }\no = C(42)\nx = o.val", "x"), 42);
}

#[test]
fn test_class_two_fields() {
    let src = r#"
class Point {
    x: Int
    y: Int
}
p = Point(3, 7)
x = p.x + p.y
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_class_with_init() {
    let src = r#"
class Doubler {
    val: Int
    init(n) { self.val = n * 2 }
}
o = Doubler(21)
x = o.val
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_with_method() {
    let src = r#"
class Counter {
    n: Int
    init(start) { self.n = start }
    def value() { return self.n }
}
c = Counter(10)
x = c.value()
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_class_method_uses_self() {
    let src = r#"
class Square {
    side: Float
    init(s) { self.side = s }
    def area() { return self.side * self.side }
}
s = Square(5.0)
x = s.area()
"#;
    assert_float(&run_get(src, "x"), 25.0);
}

#[test]
fn test_class_method_with_args() {
    let src = r#"
class Calculator {
    base: Int
    init(b) { self.base = b }
    def add(n) { return self.base + n }
}
c = Calculator(100)
x = c.add(42)
"#;
    assert_int(&run_get(src, "x"), 142);
}

#[test]
fn test_class_field_mutation() {
    let src = r#"
class Box {
    val: Int
    init(v) { self.val = v }
}
b = Box(1)
b.val = 99
x = b.val
"#;
    assert_int(&run_get(src, "x"), 99);
}

#[test]
fn test_class_mutating_method() {
    let src = r#"
class Counter {
    n: Int
    init() { self.n = 0 }
    def inc() { self.n += 1 }
}
c = Counter()
c.inc()
c.inc()
c.inc()
x = c.n
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_class_default_field_value() {
    let src = r#"
class Config {
    retries: Int = 3
    timeout: Int = 30
}
c = Config()
x = c.retries + c.timeout
"#;
    assert_int(&run_get(src, "x"), 33);
}

#[test]
fn test_class_string_field() {
    let src = r#"
class User {
    name: Str
    init(name) { self.name = name }
    def greet() { return "Hello, " + self.name }
}
u = User("Alice")
x = u.greet()
"#;
    assert_str(&run_get(src, "x"), "Hello, Alice");
}

#[test]
fn test_class_multiple_methods() {
    let src = r#"
class Rect {
    w: Float
    h: Float
    init(w, h) { self.w = w; self.h = h }
    def area() { return self.w * self.h }
    def perimeter() { return 2.0 * (self.w + self.h) }
}
r = Rect(3.0, 4.0)
x = r.area()
y = r.perimeter()
"#;
    assert_float(&run_get(src, "x"), 12.0);
    assert_float(&run_get(src, "y"), 14.0);
}

#[test]
fn test_class_many_fields() {
    let src = r#"
class Record {
    a: Int
    b: Int
    c: Int
    d: Int
    e: Int
    f: Int
    g: Int
    h: Int
    i: Int
    j: Int
}
r = Record(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
x = r.a + r.e + r.j
"#;
    assert_int(&run_get(src, "x"), 16);
}

#[test]
fn test_class_method_returns_self_field() {
    let src = r#"
class Wrapper {
    data: Int
    init(d) { self.data = d }
    def get() { return self.data }
    def set(d) { self.data = d }
}
w = Wrapper(5)
w.set(42)
x = w.get()
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_class_reference_semantics() {
    let src = r#"
class Box {
    val: Int
    init(v) { self.val = v }
}
a = Box(10)
b = a
b.val = 99
x = a.val
"#;
    assert_int(&run_get(src, "x"), 99);
}

#[test]
fn test_class_in_list() {
    let src = r#"
class Item {
    val: Int
    init(v) { self.val = v }
}
items = [Item(1), Item(2), Item(3)]
x = items[1].val
"#;
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_class_method_calling_method() {
    let src = r#"
class Calc {
    n: Int
    init(n) { self.n = n }
    def double() { return self.n * 2 }
    def quad() { return self.double() * 2 }
}
c = Calc(5)
x = c.quad()
"#;
    assert_int(&run_get(src, "x"), 20);
}

#[test]
fn test_class_conditional_method() {
    let src = r#"
class Validator {
    threshold: Int
    init(t) { self.threshold = t }
    def check(val) {
        if val > self.threshold { return "pass" }
        return "fail"
    }
}
v = Validator(10)
x = v.check(15)
y = v.check(5)
"#;
    assert_str(&run_get(src, "x"), "pass");
    assert_str(&run_get(src, "y"), "fail");
}

#[test]
fn test_class_loop_in_method() {
    let src = r#"
class Summer {
    def sum_to(n) {
        total = 0
        for i in 1..=n { total += i }
        return total
    }
}
s = Summer()
x = s.sum_to(10)
"#;
    assert_int(&run_get(src, "x"), 55);
}

#[test]
fn test_create_many_instances() {
    let src = r#"
class Widget {
    id: Int
    init(id) { self.id = id }
}
items = []
for i in 0..100 {
    items.push(Widget(i))
}
x = items.len()
"#;
    assert_int(&run_get(src, "x"), 100);
}

#[test]
fn test_class_with_bool_field() {
    let src = r#"
class Toggle {
    active: Bool
    init() { self.active = false }
    def flip() { self.active = not self.active }
}
t = Toggle()
t.flip()
x = t.active
"#;
    assert_bool(&run_get(src, "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// Inheritance (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_single_inheritance() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
}
class Dog : Animal { }
d = Dog("Rex")
x = d.name
"#;
    assert_str(&run_get(src, "x"), "Rex");
}

#[test]
fn test_inheritance_method_override() {
    let src = r#"
class Base {
    def speak() { return "base" }
}
class Child : Base {
    def speak() { return "child" }
}
x = Child().speak()
"#;
    assert_str(&run_get(src, "x"), "child");
}

#[test]
fn test_inheritance_inherits_method() {
    let src = r#"
class Base {
    def greet() { return "hello" }
}
class Child : Base { }
x = Child().greet()
"#;
    assert_str(&run_get(src, "x"), "hello");
}

#[test]
fn test_three_level_inheritance() {
    let src = r#"
class A {
    def who() { return "A" }
}
class B : A {
    def who() { return "B" }
}
class C : B {
    def who() { return "C" }
}
x = C().who()
"#;
    assert_str(&run_get(src, "x"), "C");
}

#[test]
fn test_three_level_inherited_method() {
    let src = r#"
class A {
    def base_method() { return 42 }
}
class B : A { }
class C : B { }
x = C().base_method()
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_child_accesses_parent_fields() {
    let src = r#"
class Vehicle {
    speed: Int
    init(speed) { self.speed = speed }
}
class Car : Vehicle {
    def fast() { return self.speed > 100 }
}
c = Car(120)
x = c.fast()
"#;
    assert_bool(&run_get(src, "x"), true);
}

#[test]
fn test_child_adds_own_fields() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
}
class Dog : Animal {
    tricks: Int = 0
}
d = Dog("Rex")
d.tricks = 5
x = d.tricks
y = d.name
"#;
    assert_int(&run_get(src, "x"), 5);
    assert_str(&run_get(src, "y"), "Rex");
}

#[test]
fn test_super_method_call() {
    let src = r#"
class Base {
    def value() { return 10 }
}
class Child : Base {
    def value() { return 20 }
    def base_value() { return super.value() }
}
c = Child()
x = c.value()
y = c.base_value()
"#;
    assert_int(&run_get(src, "x"), 20);
    assert_int(&run_get(src, "y"), 10);
}

#[test]
fn test_inherited_field_mutation() {
    let src = r#"
class Base {
    n: Int
    init(n) { self.n = n }
}
class Child : Base { }
c = Child(1)
c.n = 42
x = c.n
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_override_with_different_logic() {
    let src = r#"
class Shape {
    def area() { return 0.0 }
}
class Circle : Shape {
    r: Float
    init(r) { self.r = r }
    def area() { return 3.14 * self.r * self.r }
}
c = Circle(5.0)
x = c.area()
"#;
    assert_float(&run_get(src, "x"), 78.5);
}

#[test]
fn test_inheritance_with_init() {
    let src = r#"
class Person {
    name: Str
    init(name) { self.name = name }
}
class Employee : Person {
    role: Str = "worker"
}
e = Employee("Alice")
x = e.name
y = e.role
"#;
    assert_str(&run_get(src, "x"), "Alice");
    assert_str(&run_get(src, "y"), "worker");
}

#[test]
fn test_three_level_field_access() {
    let src = r#"
class A {
    val: Int
    init(v) { self.val = v }
}
class B : A { }
class C : B { }
c = C(42)
x = c.val
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_mid_level_override() {
    let src = r#"
class A {
    def f() { return 1 }
}
class B : A {
    def f() { return 2 }
}
class C : B { }
x = C().f()
"#;
    assert_int(&run_get(src, "x"), 2);
}

#[test]
fn test_child_with_own_methods() {
    let src = r#"
class Base {
    def base_fn() { return "base" }
}
class Child : Base {
    def child_fn() { return "child" }
}
c = Child()
x = c.base_fn() + "_" + c.child_fn()
"#;
    assert_str(&run_get(src, "x"), "base_child");
}

#[test]
fn test_polymorphic_list() {
    let src = r#"
class Animal {
    name: Str
    init(name) { self.name = name }
    def sound() { return "..." }
}
class Cat : Animal {
    def sound() { return "Meow" }
}
class Dog : Animal {
    def sound() { return "Woof" }
}
animals = [Cat("Whiskers"), Dog("Rex")]
x = animals[0].sound() + "_" + animals[1].sound()
"#;
    assert_str(&run_get(src, "x"), "Meow_Woof");
}

// ═══════════════════════════════════════════════════════════════
// Interfaces (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_interface_basic() {
    let src = r#"
interface Describable {
    def describe() -> Str
}
class Item impl Describable {
    name: Str
    init(n) { self.name = n }
    def describe() { return "Item: " + self.name }
}
i = Item("Widget")
x = i.describe()
"#;
    assert_str(&run_get(src, "x"), "Item: Widget");
}

#[test]
fn test_interface_with_multiple_methods() {
    let src = r#"
interface Shape {
    def area() -> Float
    def name() -> Str
}
class Square impl Shape {
    side: Float
    init(s) { self.side = s }
    def area() { return self.side * self.side }
    def name() { return "square" }
}
s = Square(4.0)
x = s.area()
y = s.name()
"#;
    assert_float(&run_get(src, "x"), 16.0);
    assert_str(&run_get(src, "y"), "square");
}

#[test]
fn test_two_classes_same_interface() {
    let src = r#"
interface Printable {
    def to_str() -> Str
}
class Num impl Printable {
    val: Int
    init(v) { self.val = v }
    def to_str() { return "num:" }
}
class Word impl Printable {
    val: Str
    init(v) { self.val = v }
    def to_str() { return "word:" }
}
a = Num(42)
b = Word("hi")
x = a.to_str() + "_" + b.to_str()
"#;
    assert_str(&run_get(src, "x"), "num:_word:");
}

#[test]
fn test_interface_method_called_on_instance() {
    let src = r#"
interface Identifiable {
    def id() -> Int
}
class User impl Identifiable {
    uid: Int
    init(uid) { self.uid = uid }
    def id() { return self.uid }
}
u = User(123)
x = u.id()
"#;
    assert_int(&run_get(src, "x"), 123);
}

#[test]
fn test_interface_impl_with_inheritance() {
    let src = r#"
interface Named {
    def get_name() -> Str
}
class Base impl Named {
    name: Str
    init(name) { self.name = name }
    def get_name() { return self.name }
}
class Child : Base { }
c = Child("test")
x = c.get_name()
"#;
    assert_str(&run_get(src, "x"), "test");
}

#[test]
fn test_interface_numeric_method() {
    let src = r#"
interface Measurable {
    def measure() -> Float
}
class Line impl Measurable {
    length: Float
    init(l) { self.length = l }
    def measure() { return self.length }
}
l = Line(3.14)
x = l.measure()
"#;
    assert_float(&run_get(src, "x"), 3.14);
}

#[test]
fn test_interface_bool_method() {
    let src = r#"
interface Checkable {
    def is_valid() -> Bool
}
class Password impl Checkable {
    len: Int
    init(len) { self.len = len }
    def is_valid() { return self.len >= 8 }
}
x = Password(10).is_valid()
y = Password(3).is_valid()
"#;
    assert_bool(&run_get(src, "x"), true);
    assert_bool(&run_get(src, "y"), false);
}

#[test]
fn test_interface_with_logic_in_method() {
    let src = r#"
interface Classifier {
    def classify() -> Str
}
class Score impl Classifier {
    val: Int
    init(v) { self.val = v }
    def classify() {
        if self.val >= 90 { return "A" }
        if self.val >= 80 { return "B" }
        return "C"
    }
}
x = Score(95).classify()
y = Score(85).classify()
z = Score(70).classify()
"#;
    assert_str(&run_get(src, "x"), "A");
    assert_str(&run_get(src, "y"), "B");
    assert_str(&run_get(src, "z"), "C");
}

#[test]
fn test_interface_with_loop_in_method() {
    let src = r#"
interface Summable {
    def total() -> Int
}
class Numbers impl Summable {
    count: Int
    init(c) { self.count = c }
    def total() {
        s = 0
        for i in 1..=self.count { s += i }
        return s
    }
}
x = Numbers(10).total()
"#;
    assert_int(&run_get(src, "x"), 55);
}

#[test]
fn test_interface_empty_with_extra_methods() {
    let src = r#"
interface Taggable {
    def tag() -> Str
}
class Doc impl Taggable {
    title: Str
    init(t) { self.title = t }
    def tag() { return "doc" }
    def full_tag() { return self.tag() + ":" + self.title }
}
d = Doc("readme")
x = d.full_tag()
"#;
    assert_str(&run_get(src, "x"), "doc:readme");
}

// ═══════════════════════════════════════════════════════════════
// Structs (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_struct_creation() {
    let src = r#"
struct Point {
    x: Float
    y: Float
}
p = Point(1.0, 2.0)
x = p.x
"#;
    assert_float(&run_get(src, "x"), 1.0);
}

#[test]
fn test_struct_field_access() {
    let src = r#"
struct Vec2 {
    x: Float
    y: Float
}
v = Vec2(3.0, 4.0)
x = v.x + v.y
"#;
    assert_float(&run_get(src, "x"), 7.0);
}

#[test]
fn test_struct_copy_semantics() {
    let src = r#"
struct Point {
    x: Float
    y: Float
}
p1 = Point(1.0, 2.0)
p2 = p1
p2.x = 99.0
x = p1.x
"#;
    assert_float(&run_get(src, "x"), 1.0);
}

#[test]
fn test_struct_field_sum() {
    let src = r#"
struct Point {
    x: Float
    y: Float
}
p = Point(3.0, 4.0)
x = p.x + p.y
"#;
    assert_float(&run_get(src, "x"), 7.0);
}

#[test]
fn test_struct_with_int_fields() {
    let src = r#"
struct Pair {
    a: Int
    b: Int
}
p = Pair(10, 20)
x = p.a + p.b
"#;
    assert_int(&run_get(src, "x"), 30);
}

#[test]
fn test_struct_mutation() {
    let src = r#"
struct Config {
    timeout: Int
    retries: Int
}
c = Config(30, 3)
c.timeout = 60
x = c.timeout
"#;
    assert_int(&run_get(src, "x"), 60);
}

#[test]
fn test_struct_in_list() {
    let src = r#"
struct Point {
    x: Int
    y: Int
}
points = [Point(1, 2), Point(3, 4), Point(5, 6)]
x = points[2].x
"#;
    assert_int(&run_get(src, "x"), 5);
}

#[test]
fn test_struct_copy_independence() {
    let src = r#"
struct Data {
    val: Int
}
a = Data(10)
b = a
c = a
b.val = 20
c.val = 30
x = a.val
"#;
    assert_int(&run_get(src, "x"), 10);
}

#[test]
fn test_struct_two_field_product() {
    let src = r#"
struct Pair {
    a: Int
    b: Int
}
p = Pair(7, 6)
x = p.a * p.b
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_struct_with_string_field() {
    let src = r#"
struct Label {
    text: Str
}
l = Label("hello")
x = l.text
"#;
    assert_str(&run_get(src, "x"), "hello");
}

// ═══════════════════════════════════════════════════════════════
// Operator overloading (10 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_operator_add() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator +(other: Vec2) -> Vec2 {
        return Vec2(self.x + other.x, self.y + other.y)
    }
}
a = Vec2(1.0, 2.0)
b = Vec2(3.0, 4.0)
c = a + b
x = c.x
y = c.y
"#;
    assert_float(&run_get(src, "x"), 4.0);
    assert_float(&run_get(src, "y"), 6.0);
}

#[test]
fn test_operator_eq_true() {
    let src = r#"
class Point {
    x: Int
    y: Int
    init(x, y) { self.x = x; self.y = y }
    operator ==(other: Point) -> Bool {
        return self.x == other.x and self.y == other.y
    }
}
a = Point(1, 2)
b = Point(1, 2)
x = a == b
"#;
    assert_bool(&run_get(src, "x"), true);
}

#[test]
fn test_operator_eq_false() {
    let src = r#"
class Point {
    x: Int
    y: Int
    init(x, y) { self.x = x; self.y = y }
    operator ==(other: Point) -> Bool {
        return self.x == other.x and self.y == other.y
    }
}
a = Point(1, 2)
b = Point(3, 4)
x = a == b
"#;
    assert_bool(&run_get(src, "x"), false);
}

#[test]
fn test_operator_sub() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator -(other: Vec2) -> Vec2 {
        return Vec2(self.x - other.x, self.y - other.y)
    }
}
a = Vec2(10.0, 20.0)
b = Vec2(3.0, 5.0)
c = a - b
x = c.x
"#;
    assert_float(&run_get(src, "x"), 7.0);
}

#[test]
fn test_operator_mul() {
    let src = r#"
class Scale {
    val: Float
    init(v) { self.val = v }
    operator *(other: Scale) -> Scale {
        return Scale(self.val * other.val)
    }
}
a = Scale(3.0)
b = Scale(4.0)
c = a * b
x = c.val
"#;
    assert_float(&run_get(src, "x"), 12.0);
}

#[test]
fn test_chained_operator_add() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator +(other: Vec2) -> Vec2 {
        return Vec2(self.x + other.x, self.y + other.y)
    }
}
a = Vec2(1.0, 1.0)
b = Vec2(2.0, 2.0)
c = Vec2(3.0, 3.0)
d = a + b + c
x = d.x
"#;
    assert_float(&run_get(src, "x"), 6.0);
}

#[test]
fn test_operator_ne() {
    let src = r#"
class Point {
    x: Int
    y: Int
    init(x, y) { self.x = x; self.y = y }
    operator !=(other: Point) -> Bool {
        return self.x != other.x or self.y != other.y
    }
}
a = Point(1, 2)
b = Point(3, 4)
x = a != b
"#;
    assert_bool(&run_get(src, "x"), true);
}

#[test]
fn test_operator_add_with_int_class() {
    let src = r#"
class Money {
    cents: Int
    init(c) { self.cents = c }
    operator +(other: Money) -> Money {
        return Money(self.cents + other.cents)
    }
}
a = Money(150)
b = Money(250)
c = a + b
x = c.cents
"#;
    assert_int(&run_get(src, "x"), 400);
}

#[test]
fn test_operator_triple_chain() {
    let src = r#"
class Money {
    cents: Int
    init(c) { self.cents = c }
    operator +(other: Money) -> Money {
        return Money(self.cents + other.cents)
    }
}
total = Money(100) + Money(200) + Money(300) + Money(400)
x = total.cents
"#;
    assert_int(&run_get(src, "x"), 1000);
}

#[test]
fn test_operator_in_method() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    operator +(other: Vec2) -> Vec2 {
        return Vec2(self.x + other.x, self.y + other.y)
    }
    def mag_sq() { return self.x * self.x + self.y * self.y }
}
a = Vec2(3.0, 0.0)
b = Vec2(0.0, 4.0)
c = a + b
x = c.mag_sq()
"#;
    assert_float(&run_get(src, "x"), 25.0);
}

// ═══════════════════════════════════════════════════════════════
// Closures and lambdas (15 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_lambda_basic() {
    assert_int(&run_get("f = x -> x * 2\nx = f(21)", "x"), 42);
}

#[test]
fn test_lambda_multi_param() {
    assert_int(&run_get("f = (a, b) -> a + b\nx = f(10, 32)", "x"), 42);
}

#[test]
fn test_lambda_as_argument() {
    assert_int(&run_get("x = [1, 2, 3, 4].filter(x -> x > 2).len()", "x"), 2);
}

#[test]
fn test_lambda_map() {
    assert_int(&run_get("x = [1, 2, 3].map(x -> x * 10).sum()", "x"), 60);
}

#[test]
fn test_lambda_chain_map_filter() {
    // Double each, then filter > 4: [2,4,6] -> filter > 4 -> [6] -> len=1
    assert_int(&run_get("x = [1, 2, 3].map(x -> x * 2).filter(x -> x > 4).len()", "x"), 1);
}

#[test]
fn test_closure_captures_variable() {
    let src = r#"
n = 10
f = x -> x + n
x = f(32)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_closure_returned_from_function() {
    let src = r#"
def make_adder(n) {
    return x -> x + n
}
add10 = make_adder(10)
x = add10(32)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_closure_factory_multiple() {
    let src = r#"
def make_multiplier(n) {
    return x -> x * n
}
double = make_multiplier(2)
triple = make_multiplier(3)
x = double(5) + triple(5)
"#;
    assert_int(&run_get(src, "x"), 25);
}

#[test]
fn test_lambda_identity() {
    assert_int(&run_get("f = x -> x\nx = f(42)", "x"), 42);
}

#[test]
fn test_lambda_with_string() {
    assert_str(&run_get("f = x -> x + \"!\"\nx = f(\"hi\")", "x"), "hi!");
}

#[test]
fn test_lambda_boolean() {
    assert_bool(&run_get("f = x -> x > 10\nx = f(15)", "x"), true);
    assert_bool(&run_get("f = x -> x > 10\nx = f(5)", "x"), false);
}

#[test]
fn test_higher_order_apply() {
    let src = r#"
def apply(f, val) {
    return f(val)
}
x = apply(x -> x * 3, 14)
"#;
    assert_int(&run_get(src, "x"), 42);
}

#[test]
fn test_higher_order_compose() {
    let src = r#"
def compose(f, g) {
    return x -> f(g(x))
}
double = x -> x * 2
inc = x -> x + 1
double_then_inc = compose(inc, double)
x = double_then_inc(20)
"#;
    assert_int(&run_get(src, "x"), 41);
}

#[test]
fn test_lambda_reduce() {
    assert_int(&run_get("x = [1, 2, 3, 4, 5].reduce(0, (acc, v) -> acc + v)", "x"), 15);
}

#[test]
fn test_lambda_reduce_product() {
    assert_int(&run_get("x = [1, 2, 3, 4].reduce(1, (acc, v) -> acc * v)", "x"), 24);
}

// ═══════════════════════════════════════════════════════════════
// Computed properties (5 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_computed_property_basic() {
    let src = r#"
class Rect {
    w: Float
    h: Float
    area: Float => self.w * self.h
    init(w, h) { self.w = w; self.h = h }
}
r = Rect(5.0, 3.0)
x = r.area
"#;
    assert_float(&run_get(src, "x"), 15.0);
}

#[test]
fn test_computed_property_updates_on_change() {
    let src = r#"
class Rect {
    w: Float
    h: Float
    area: Float => self.w * self.h
    init(w, h) { self.w = w; self.h = h }
}
r = Rect(3.0, 4.0)
r.w = 10.0
x = r.area
"#;
    assert_float(&run_get(src, "x"), 40.0);
}

#[test]
fn test_computed_property_with_method() {
    let src = r#"
class Circle {
    r: Float
    circumference: Float => 2.0 * 3.14159 * self.r
    init(r) { self.r = r }
}
c = Circle(1.0)
x = c.circumference
"#;
    assert_float(&run_get(src, "x"), 6.28318);
}

#[test]
fn test_computed_property_int() {
    let src = r#"
class Range {
    low: Int
    high: Int
    span: Int => self.high - self.low
    init(l, h) { self.low = l; self.high = h }
}
r = Range(10, 50)
x = r.span
"#;
    assert_int(&run_get(src, "x"), 40);
}

#[test]
fn test_computed_property_bool() {
    let src = r#"
class Account {
    balance: Float
    overdrawn: Bool => self.balance < 0.0
    init(b) { self.balance = b }
}
a = Account(-5.0)
x = a.overdrawn
"#;
    assert_bool(&run_get(src, "x"), true);
}

// ═══════════════════════════════════════════════════════════════
// Mixed OOP scenarios (5 tests)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_class_with_list_field() {
    let src = r#"
class Stack {
    items: List
    init() { self.items = [] }
    def push(v) { self.items.push(v) }
    def size() { return self.items.len() }
}
s = Stack()
s.push(1)
s.push(2)
s.push(3)
x = s.size()
"#;
    assert_int(&run_get(src, "x"), 3);
}

#[test]
fn test_class_with_map_field() {
    let src = r#"
class Registry {
    data: Map
    init() { self.data = {} }
    def set(key, val) { self.data[key] = val }
    def get(key) { return self.data[key] }
}
r = Registry()
r.set("name", "Alice")
x = r.get("name")
"#;
    assert_str(&run_get(src, "x"), "Alice");
}

#[test]
fn test_class_using_another_class() {
    let src = r#"
class Point {
    x: Int
    y: Int
    init(x, y) { self.x = x; self.y = y }
}
class Line {
    start: Point
    end_pt: Point
    init(s, e) { self.start = s; self.end_pt = e }
    def dx() { return self.end_pt.x - self.start.x }
}
l = Line(Point(1, 2), Point(5, 10))
x = l.dx()
"#;
    assert_int(&run_get(src, "x"), 4);
}

#[test]
fn test_method_returns_new_instance() {
    let src = r#"
class Vec2 {
    x: Float
    y: Float
    init(x, y) { self.x = x; self.y = y }
    def scale(factor) { return Vec2(self.x * factor, self.y * factor) }
}
v = Vec2(3.0, 4.0)
v2 = v.scale(2.0)
x = v2.x
"#;
    assert_float(&run_get(src, "x"), 6.0);
}

#[test]
fn test_iterator_pattern() {
    let src = r#"
class Range {
    current: Int
    max: Int
    init(max) { self.current = 0; self.max = max }
    def has_next() { return self.current < self.max }
    def next_val() {
        val = self.current
        self.current += 1
        return val
    }
}
r = Range(5)
total = 0
loop while r.has_next() {
    total += r.next_val()
}
x = total
"#;
    assert_int(&run_get(src, "x"), 10);
}
