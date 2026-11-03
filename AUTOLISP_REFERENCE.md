# OpenCAD Studio AutoLISP Function Reference

This reference documents every built-in function, special form, CAD bridge operation, Visual LISP extension, and native UI constructor implemented in the **opencad-lisp** engine.

**Argument Conventions:**
- `arg` — Required argument.
- `[arg]` — Optional argument.
- `arg1 ...` — Zero or more additional arguments (variadic).

---

## 1. Special Forms & Control Flow

### `setq`
- **Syntax**: `(setq sym1 expr1 [sym2 expr2 ...])`
- **Summary**: Assigns the result of each expression to its preceding symbol sequentially and returns the last assigned value.
- **Example**:
  ```lisp
  (setq radius 25.0
        area (* pi radius radius))
  ```

### `set`
- **Syntax**: `(set sym-expr val-expr)`
- **Summary**: Evaluates `sym-expr` to obtain a symbol and binds it to the evaluated `val-expr`.
- **Example**:
  ```lisp
  (setq var-name 'my-var)
  (set var-name 42) ; sets my-var to 42
  ```

### `quote` / `'`
- **Syntax**: `(quote expr)` or `'expr`
- **Summary**: Returns `expr` unevaluated.
- **Example**:
  ```lisp
  (setq pts '((0 0 0) (10 10 0)))
  ```

### `defun`
- **Syntax**: `(defun name ([param1 ...] [/ local1 ...]) expr1 ...)`
- **Summary**: Defines a function or CAD command. Prefixing `name` with `c:` (e.g., `c:mycmd`) automatically registers `MYCMD` as a top-level command in OpenCAD Studio. Symbols after `/` are scoped as local variables initialized to `nil`.
- **Example**:
  ```lisp
  (defun area-circle (r / a)
    (setq a (* pi r r))
    a
  )
  (defun c:drawbox ()
    (command "_.LINE" '(0 0) '(10 0) '(10 10) '(0 10) "C")
  )
  ```

### `lambda`
- **Syntax**: `(lambda ([param1 ...] [/ local1 ...]) expr1 ...)`
- **Summary**: Creates an anonymous function closure, commonly used with `mapcar`, `apply`, or `vl-remove-if`.
- **Example**:
  ```lisp
  (mapcar '(lambda (x) (* x 2)) '(1 2 3 4)) ; => (2 4 6 8)
  ```

### `if`
- **Syntax**: `(if test-expr then-expr [else-expr])`
- **Summary**: Evaluates `test-expr`. If non-`nil`, evaluates and returns `then-expr`; otherwise evaluates and returns `else-expr` (or `nil` if omitted).
- **Example**:
  ```lisp
  (if (> x 0) "Positive" "Non-positive")
  ```

### `cond`
- **Syntax**: `(cond (test1 [expr1 ...]) [(test2 [expr2 ...]) ...])`
- **Summary**: Evaluates each test clause in order until one evaluates to non-`nil`, then evaluates the body expressions of that clause and returns the last result.
- **Example**:
  ```lisp
  (cond
    ((= color "Red") 1)
    ((= color "Green") 3)
    (t 7)
  )
  ```

### `while`
- **Syntax**: `(while test-expr [body-expr1 ...])`
- **Summary**: Repeatedly evaluates the body expressions as long as `test-expr` evaluates to non-`nil`. Returns the last evaluated expression.
- **Example**:
  ```lisp
  (setq i 0)
  (while (< i 5)
    (princ i)
    (setq i (1+ i))
  )
  ```

### `repeat`
- **Syntax**: `(repeat count [body-expr1 ...])`
- **Summary**: Evaluates the body expressions `count` times (where `count` is a non-negative integer) and returns the result of the final expression.
- **Example**:
  ```lisp
  (setq total 0)
  (repeat 4 (setq total (+ total 10))) ; => 40
  ```

### `foreach`
- **Syntax**: `(foreach var list [body-expr1 ...])`
- **Summary**: Iterates through every element in `list`, binding `var` to each item and evaluating the body expressions.
- **Example**:
  ```lisp
  (foreach pt '((0 0) (10 0) (10 10))
    (print pt)
  )
  ```

### `progn`
- **Syntax**: `(progn [expr1 ...])`
- **Summary**: Evaluates multiple expressions sequentially as a single block and returns the result of the last expression.
- **Example**:
  ```lisp
  (if draw-x
    (progn
      (princ "\nDrawing X...")
      (command "_.LINE" '(0 0) '(10 10) "")
    )
  )
  ```

### `and`
- **Syntax**: `(and [expr1 ...])`
- **Summary**: Logical short-circuit AND. Evaluates expressions left-to-right; stops and returns `nil` at the first `nil` expression, or `T`/truthy if all succeed.
- **Example**:
  ```lisp
  (and pt (> (car pt) 0.0))
  ```

### `or`
- **Syntax**: `(or [expr1 ...])`
- **Summary**: Logical short-circuit OR. Evaluates expressions left-to-right and returns the first non-`nil` value, or `nil` if all are `nil`.
- **Example**:
  ```lisp
  (or (= ans "Y") (= ans "y"))
  ```

### `eval`
- **Syntax**: `(eval expr)`
- **Summary**: Evaluates `expr` a second time as an AutoLISP expression.
- **Example**:
  ```lisp
  (setq code '(+ 10 20))
  (eval code) ; => 30
  ```

### `boundp`
- **Syntax**: `(boundp sym)`
- **Summary**: Returns `T` if the symbol is bound to a non-`nil` value in the current environment, or `nil` otherwise.
- **Example**:
  ```lisp
  (boundp 'pi) ; => T
  ```

---

## 2. Predefined Constants & System Info

### `T` / `nil` / `pi` / `PAUSE`
- **Summary**:
  - `T`: Boolean true constant.
  - `nil`: Boolean false / empty list constant.
  - `pi`: Mathematical constant pi (approx. 3.141592653589793).
  - `PAUSE`: Pause token (`"\\"`) used in `(command ...)` calls for interactive user input.
- **Example**:
  ```lisp
  (setq circumference (* 2.0 pi 25.0))
  (command "_.CIRCLE" PAUSE 25.0)
  ```

### `ver` / `vl-version` / `lisp-implementation-type`
- **Syntax**: `(ver)`, `(vl-version)`, `(lisp-implementation-type)`
- **Summary**: Returns version and implementation strings for the AutoLISP runtime.
- **Example**:
  ```lisp
  (ver) ; => "Visual LISP 2026 (opencad-lisp native Rust engine v0.1.0)"
  ```

---

## 3. Math, Bitwise & Geometry Functions

### `+`
- **Syntax**: `(+ [num1 num2 ...])`
- **Summary**: Returns the sum of all numbers. Returns `0` if no arguments are supplied. Promotes to `Real` if any argument is a float.
- **Example**:
  ```lisp
  (+ 10 20 5.5) ; => 35.5
  ```

### `-`
- **Syntax**: `(- [num1 num2 ...])`
- **Summary**: With one argument, negates `num1`. With multiple arguments, subtracts `num2 ...` from `num1`.
- **Example**:
  ```lisp
  (- 50 15 5) ; => 30
  (- 12)      ; => -12
  ```

### `*`
- **Syntax**: `(* [num1 num2 ...])`
- **Summary**: Returns the product of all numbers. Returns `1` if no arguments are given.
- **Example**:
  ```lisp
  (* 4 5 2.0) ; => 40.0
  ```

### `/`
- **Syntax**: `(/ [num1 num2 ...])`
- **Summary**: Divides `num1` by `num2` and any subsequent divisors. Integer operands perform integer division; include a float for decimal division.
- **Example**:
  ```lisp
  (/ 100.0 4 5) ; => 5.0
  ```

### `1+` / `1-`
- **Syntax**: `(1+ num)` / `(1- num)`
- **Summary**: Increments (`1+`) or decrements (`1-`) `num` by 1.
- **Example**:
  ```lisp
  (1+ 9) ; => 10
  (1- 9) ; => 8
  ```

### `abs`
- **Syntax**: `(abs num)`
- **Summary**: Returns the absolute value of `num`.
- **Example**:
  ```lisp
  (abs -42.5) ; => 42.5
  ```

### `max` / `min`
- **Syntax**: `(max num1 [num2 ...])` / `(min num1 [num2 ...])`
- **Summary**: Returns the largest (`max`) or smallest (`min`) of the given numbers.
- **Example**:
  ```lisp
  (max 3 18 -2 7) ; => 18
  (min 3 18 -2 7) ; => -2
  ```

### `rem`
- **Syntax**: `(rem num1 num2 [num3 ...])`
- **Summary**: Divides `num1` by `num2` (and subsequent divisors) and returns the remainder.
- **Example**:
  ```lisp
  (rem 17 5) ; => 2
  ```

### `gcd`
- **Syntax**: `(gcd int1 int2)`
- **Summary**: Returns the greatest common divisor of two integers.
- **Example**:
  ```lisp
  (gcd 84 36) ; => 12
  ```

### `sqrt`
- **Syntax**: `(sqrt num)`
- **Summary**: Returns the square root of a non-negative number as a real.
- **Example**:
  ```lisp
  (sqrt 2.0) ; => 1.4142135623730951
  ```

### `expt`
- **Syntax**: `(expt base power)`
- **Summary**: Raises `base` to the specified `power`.
- **Example**:
  ```lisp
  (expt 2 10) ; => 1024
  ```

### `sin` / `cos`
- **Syntax**: `(sin ang)` / `(cos ang)`
- **Summary**: Returns the sine or cosine of `ang` (specified in radians).
- **Example**:
  ```lisp
  (cos 0.0)        ; => 1.0
  (sin (/ pi 2.0)) ; => 1.0
  ```

### `atan`
- **Syntax**: `(atan num1 [num2])`
- **Summary**: Returns the arctangent of `num1` in radians, or `atan2(num1, num2)` if `num2` is provided.
- **Example**:
  ```lisp
  (atan 1.0 1.0) ; => 0.7853981633974483
  ```

### `log` / `exp`
- **Syntax**: `(log num)` / `(exp num)`
- **Summary**: `log` returns the natural logarithm (ln) of `num`. `exp` returns e raised to the power of `num`.
- **Example**:
  ```lisp
  (log (exp 1.0)) ; => 1.0
  ```

### `float` / `fix`
- **Syntax**: `(float num)` / `(fix num)`
- **Summary**: `float` converts a number to a floating-point `Real`. `fix` truncates a number toward zero to an `Int`.
- **Example**:
  ```lisp
  (float 5)  ; => 5.0
  (fix 3.95) ; => 3
  ```

### `logand` / `logior` / `~` / `lsh` / `boole`
- **Syntax**:
  - `(logand int1 int2 ...)`
  - `(logior int1 int2 ...)`
  - `(~ int)`
  - `(lsh int bits)`
  - `(boole op int1 int2 ...)`
- **Summary**: Bitwise integer operations: AND (`logand`), OR (`logior`), bitwise NOT (`~`), logical bit shift (`lsh`), and general bitwise operator (`boole`, where `1`=AND, `6`=XOR, `7`=OR, `8`=NOR).
- **Example**:
  ```lisp
  (logand 7 3)    ; => 3
  (logior 4 2)    ; => 6
  (lsh 1 4)       ; => 16
  (boole 6 12 10) ; => 6
  ```

### `distance`
- **Syntax**: `(distance pt1 pt2)`
- **Summary**: Returns the 2D or 3D Euclidean distance between points `pt1` and `pt2`.
- **Example**:
  ```lisp
  (distance '(0 0 0) '(3 4 0)) ; => 5.0
  ```

### `angle`
- **Syntax**: `(angle pt1 pt2)`
- **Summary**: Returns the angle in radians in the XY plane from `pt1` to `pt2` measured counter-clockwise from the positive X axis.
- **Example**:
  ```lisp
  (angle '(0 0) '(0 10)) ; => 1.5707963267948966
  ```

### `polar`
- **Syntax**: `(polar pt ang dist)`
- **Summary**: Returns a new 3D point displaced from `pt` at angle `ang` (in radians in the XY plane) by distance `dist`.
- **Example**:
  ```lisp
  (polar '(10.0 10.0 0.0) 0.0 25.0) ; => (35.0 10.0 0.0)
  ```

### `inters`
- **Syntax**: `(inters pt1 pt2 pt3 pt4 [onseg])`
- **Summary**: Computes the intersection point of line `(pt1, pt2)` and line `(pt3, pt4)`. If `onseg` is omitted or non-`nil`, the intersection must lie on both finite segments; if `nil`, the lines are treated as infinite.
- **Example**:
  ```lisp
  (inters '(0 0) '(10 10) '(0 10) '(10 0)) ; => (5.0 5.0 0.0)
  ```

---

## 4. Comparison, Logic & Type Predicates

### `=` / `/=` / `<` / `<=` / `>` / `>=`
- **Syntax**: `(= a b [c ...])`, `(/= a b)`, `(< a b [c ...])`, `(<= a b [c ...])`, `(> a b [c ...])`, `(>= a b [c ...])`
- **Summary**: Compares numbers or strings. Returns `T` if the relation holds across arguments, or `nil` otherwise.
- **Example**:
  ```lisp
  (< 1 5 10)        ; => T
  (/= "Red" "Blue") ; => T
  ```

### `eq` / `equal`
- **Syntax**: `(eq expr1 expr2)` / `(equal expr1 expr2 [fuzz])`
- **Summary**: Checks equality of two expressions. `equal` performs deep structural comparison and accepts an optional numeric `fuzz` tolerance.
- **Example**:
  ```lisp
  (equal 10.0001 10.0 0.001) ; => T
  ```

### `not` / `null`
- **Syntax**: `(not expr)` / `(null expr)`
- **Summary**: Returns `T` if `expr` is `nil`, or `nil` otherwise.
- **Example**:
  ```lisp
  (null nil) ; => T
  ```

### `atom` / `listp` / `vl-consp`
- **Syntax**: `(atom expr)` / `(listp expr)` / `(vl-consp expr)`
- **Summary**: `atom` returns `T` if `expr` is not a cons cell. `listp` returns `T` for lists or `nil`. `vl-consp` returns `T` only for non-empty lists.
- **Example**:
  ```lisp
  (listp '(1 2 3)) ; => T
  (vl-consp nil)   ; => nil
  ```

### `numberp` / `minusp` / `zerop`
- **Syntax**: `(numberp expr)` / `(minusp num)` / `(zerop num)`
- **Summary**: `numberp` returns `T` if `expr` is an integer or real. `minusp` tests if `num < 0`. `zerop` tests if `num == 0`.
- **Example**:
  ```lisp
  (numberp 3.14) ; => T
  (minusp -5)    ; => T
  (zerop 0.0)    ; => T
  ```

### `type`
- **Syntax**: `(type expr)`
- **Summary**: Returns a symbol representing the data type of `expr` (`INT`, `REAL`, `STR`, `SYM`, `LIST`, `SUBR`, `USUBR`, `ENAME`, `PICKSET`, `VLA-OBJECT`, `FILE`, or `nil`).
- **Example**:
  ```lisp
  (type 42)     ; => INT
  (type "text") ; => STR
  ```

---

## 5. List Manipulation & Higher-Order Functions

### `cons`
- **Syntax**: `(cons first rest)`
- **Summary**: Constructs a pair/list by prepending `first` onto `rest` (or creating a dotted pair `(a . b)` when `rest` is an atom).
- **Example**:
  ```lisp
  (cons 1 '(2 3)) ; => (1 2 3)
  (cons 62 1)     ; => (62 . 1)
  ```

### `list`
- **Syntax**: `(list [expr1 ...])`
- **Summary**: Creates a list containing all evaluated arguments.
- **Example**:
  ```lisp
  (list 10.0 20.0 0.0) ; => (10.0 20.0 0.0)
  ```

### `append`
- **Syntax**: `(append [list1 list2 ...])`
- **Summary**: Concatenates multiple lists into a single flat list.
- **Example**:
  ```lisp
  (append '(1 2) '(3 4)) ; => (1 2 3 4)
  ```

### `reverse`
- **Syntax**: `(reverse list)`
- **Summary**: Returns a new list with the top-level elements in reverse order.
- **Example**:
  ```lisp
  (reverse '(1 2 3)) ; => (3 2 1)
  ```

### `length`
- **Syntax**: `(length list)`
- **Summary**: Returns the number of elements in `list` as an integer.
- **Example**:
  ```lisp
  (length '(a b c d)) ; => 4
  ```

### `nth`
- **Syntax**: `(nth index list)`
- **Summary**: Returns the 0-indexed element at `index` of `list` (or `nil` if out of bounds).
- **Example**:
  ```lisp
  (nth 1 '("Red" "Green" "Blue")) ; => "Green"
  ```

### `last`
- **Syntax**: `(last list)`
- **Summary**: Returns the last element in `list`.
- **Example**:
  ```lisp
  (last '(10 20 30)) ; => 30
  ```

### `car` / `cdr` & `caar` .. `cddddr`
- **Syntax**: `(car list)`, `(cdr list)`, `(cadr list)`, `(caddr list)`, etc.
- **Summary**: Extracts the head (`car`), tail (`cdr`), X-coordinate (`car`), Y-coordinate (`cadr`), Z-coordinate (`caddr`), and all combinations up to 4 levels deep (`caar` through `cddddr`).
- **Example**:
  ```lisp
  (setq pt '(15.0 25.0 5.0))
  (car pt)   ; => 15.0
  (cadr pt)  ; => 25.0
  (caddr pt) ; => 5.0
  ```

### `assoc`
- **Syntax**: `(assoc key alist)`
- **Summary**: Searches an association list `alist` for a pair whose `car` equals `key` and returns that pair, or `nil` if not found.
- **Example**:
  ```lisp
  (assoc 40 '((0 . "CIRCLE") (40 . 25.0))) ; => (40 . 25.0)
  ```

### `member`
- **Syntax**: `(member expr list)`
- **Summary**: Searches `list` for `expr` and returns the sublist starting at the matching element, or `nil` if not found.
- **Example**:
  ```lisp
  (member 'b '(a b c d)) ; => (B C D)
  ```

### `subst`
- **Syntax**: `(subst new-item old-item list)`
- **Summary**: Returns a copy of `list` with every occurrence of `old-item` replaced by `new-item`.
- **Example**:
  ```lisp
  (subst '(40 . 50.0) '(40 . 25.0) '((0 . "CIRCLE") (40 . 25.0)))
  ```

### `apply`
- **Syntax**: `(apply func arg-list)`
- **Summary**: Invokes `func` (a symbol or lambda) using the elements of `arg-list` as its arguments.
- **Example**:
  ```lisp
  (apply '+ '(10 20 30)) ; => 60
  ```

### `mapcar`
- **Syntax**: `(mapcar func list1 [list2 ...])`
- **Summary**: Applies `func` element-wise across one or more lists and returns a list of the results.
- **Example**:
  ```lisp
  (mapcar '+ '(1 2 3) '(10 20 30)) ; => (11 22 33)
  ```

---

## 6. String Manipulation & Conversions

### `strcat`
- **Syntax**: `(strcat [str1 str2 ...])`
- **Summary**: Concatenates multiple strings into a single string.
- **Example**:
  ```lisp
  (strcat "Layer: " "SHAPE-RED") ; => "Layer: SHAPE-RED"
  ```

### `strlen`
- **Syntax**: `(strlen [str1 ...])`
- **Summary**: Returns the total character count of the supplied string(s).
- **Example**:
  ```lisp
  (strlen "OpenCAD") ; => 7
  ```

### `substr`
- **Syntax**: `(substr str start [length])`
- **Summary**: Extracts a substring from `str` starting at 1-based index `start` for `length` characters (or to the end of the string if `length` is omitted).
- **Example**:
  ```lisp
  (substr "OpenCAD Studio" 1 7) ; => "OpenCAD"
  ```

### `strcase`
- **Syntax**: `(strcase str [which])`
- **Summary**: Converts `str` to uppercase (default when `which` is `nil` or omitted) or lowercase (when `which` is non-`nil`).
- **Example**:
  ```lisp
  (strcase "Hello")   ; => "HELLO"
  (strcase "Hello" T) ; => "hello"
  ```

### `itoa` / `atoi` / `atof`
- **Syntax**: `(itoa int)`, `(atoi str)`, `(atof str)`
- **Summary**: Converts an integer to a string (`itoa`), a string to an integer (`atoi`), or a string to a real float (`atof`).
- **Example**:
  ```lisp
  (itoa 42)     ; => "42"
  (atof "3.14") ; => 3.14
  ```

### `rtos`
- **Syntax**: `(rtos num [mode [precision]])`
- **Summary**: Formats a real number as a string. `mode`: `1` = scientific, `2` = decimal (default). `precision` sets decimal places (default `4`).
- **Example**:
  ```lisp
  (rtos 25.1256 2 2) ; => "25.13"
  ```

### `angtos` / `angtof` / `distof`
- **Syntax**:
  - `(angtos ang [mode [precision]])`
  - `(angtof str [mode])`
  - `(distof str [mode])`
- **Summary**: Converts angles in radians to formatted strings (`angtos`), angle strings to radians (`angtof`), and distance strings to real numbers (`distof`). Modes: `0`=degrees, `1`=DMS, `2`=grads, `3`=radians.
- **Example**:
  ```lisp
  (angtos pi 0 1) ; => "180.0"
  (angtof "90" 0) ; => 1.5707963267948966
  ```

### `ascii` / `chr`
- **Syntax**: `(ascii str)` / `(chr int)`
- **Summary**: `ascii` returns the ASCII code of the first character of `str`. `chr` converts an ASCII code to a single-character string.
- **Example**:
  ```lisp
  (ascii "A") ; => 65
  (chr 65)    ; => "A"
  ```

### `read`
- **Syntax**: `(read [str])`
- **Summary**: Parses the first Lisp expression from `str` and returns the corresponding data structure.
- **Example**:
  ```lisp
  (read "(10 20 30)") ; => (10 20 30)
  ```

### `wcmatch`
- **Syntax**: `(wcmatch str pattern)`
- **Summary**: Performs wildcard pattern matching (`*` matches any sequence, `?` matches any single character, `#` matches any digit, `,` separates multiple patterns).
- **Example**:
  ```lisp
  (wcmatch "SHAPE-RED" "SHAPE-*") ; => T
  ```

---

## 7. Console Output, File I/O & Environment

### `princ` / `prin1` / `print` / `prompt`
- **Syntax**:
  - `(princ [expr [file-desc]])`
  - `(prin1 [expr [file-desc]])`
  - `(print [expr [file-desc]])`
  - `(prompt msg)`
- **Summary**: Writes output to the OpenCAD Studio command line (or `file-desc`). `princ` prints strings unquoted; `prin1` prints strings escaped; `print` prints a newline followed by `prin1` and a space. Calling `(princ)` with no arguments suppresses the final return value echo.
- **Example**:
  ```lisp
  (princ "\nCalculation complete.")
  (princ)
  ```

### `open` / `close`
- **Syntax**: `(open filename mode)` / `(close file-desc)`
- **Summary**: Opens a file for reading (`"r"`), writing (`"w"`), or appending (`"a"`), returning a file descriptor. `close` closes the descriptor.
- **Example**:
  ```lisp
  (setq f (open "output.txt" "w"))
  (write-line "Hello CAD" f)
  (close f)
  ```

### `read-line` / `write-line` / `read-char` / `write-char`
- **Syntax**:
  - `(read-line [file-desc])`
  - `(write-line str [file-desc])`
  - `(read-char [file-desc])`
  - `(write-char ascii-code [file-desc])`
- **Summary**: Reads or writes lines and character codes to/from an open file descriptor or standard stream.
- **Example**:
  ```lisp
  (setq line (read-line f))
  ```

### `findfile`
- **Syntax**: `(findfile filename)`
- **Summary**: Searches for `filename` and returns its full canonical path if it exists, or `nil` if not found.
- **Example**:
  ```lisp
  (findfile "routine.lsp")
  ```

### `load`
- **Syntax**: `(load filename [onfailure])`
- **Summary**: Reads and evaluates an AutoLISP `.lsp` file. Automatically appends `.lsp` if omitted. If loading fails and `onfailure` is provided, returns `onfailure`.
- **Example**:
  ```lisp
  (load "routine")
  ```

### `getenv` / `setenv`
- **Syntax**: `(getenv var-name)` / `(setenv var-name value)`
- **Summary**: Reads or sets an operating system environment variable.
- **Example**:
  ```lisp
  (getenv "APPDATA")
  ```

---

## 8. Clipboard & DOSLib Extensions

### `get_clipboard` / `dos_getclipboard`
- **Syntax**: `(get_clipboard)` or `(dos_getclipboard)`
- **Summary**: Retrieves the current plain-text contents of the system clipboard.
- **Example**:
  ```lisp
  (setq text (dos_getclipboard))
  ```

### `set_clipboard` / `dos_setclipboard`
- **Syntax**: `(set_clipboard val)` or `(dos_setclipboard val)`
- **Summary**: Copies `val` (converted to string) to the system clipboard and returns the string.
- **Example**:
  ```lisp
  (dos_setclipboard "42.50")
  ```

### `dos_clipboard`
- **Syntax**: `(dos_clipboard [new-text])`
- **Summary**: Unified getter/setter. With no arguments, returns current clipboard text; with `new-text`, sets the clipboard and returns the new text.
- **Example**:
  ```lisp
  (dos_clipboard "Copied from AutoLISP")
  ```

### `msgbox` / `dos_msgbox` / `ocl-msgbox`
- **Syntax**: `(ocl-msgbox prompt [title] [buttons])` *(aliases: `msgbox`, `dos_msgbox`)*
- **Summary**: Displays a modal message box dialog. `buttons` can be an integer or symbol (`0`/`"OK"`, `1`/`"OKCANCEL"`, `4`/`"YESNO"`, `5`/`"RETRYCANCEL"`). Returns standard button ID (`1`=OK, `2`=Cancel, `4`=Retry, `6`=Yes, `7`=No).
- **Example**:
  ```lisp
  (ocl-msgbox "Delete selected entities?" "Confirm" "YESNO") ; => 6 (Yes) or 7 (No)
  ```

---

## 9. CAD Command Execution & System Variables

### `command`
- **Syntax**: `(command [cmd-name arg1 ...])`
- **Summary**: Executes native OpenCAD Studio commands (`_.LINE`, `_.CIRCLE`, `_.POINT`, `_.COLOR`, etc.). Supports streaming multi-call sequences, 2D/3D point lists, `"C"` (close), `""` (enter), and `PAUSE` for interactive user picks.
- **Example**:
  ```lisp
  (command "_.COLOR" "RED")
  (command "_.CIRCLE" '(0 0 0) 25.0)
  (command "_.LINE" '(0 0 0) '(25 25 0) "")
  ```

### `getvar` / `setvar`
- **Syntax**: `(getvar var-name)` / `(setvar var-name value)`
- **Summary**: Reads or updates an OpenCAD Studio system variable (e.g., `"CLAYER"`, `"OSMODE"`).
- **Example**:
  ```lisp
  (getvar "CLAYER")
  (setvar "CLAYER" "0")
  ```

### `alert`
- **Syntax**: `(alert msg)`
- **Summary**: Displays an informational message to the user in the CAD host.
- **Example**:
  ```lisp
  (alert "Operation completed successfully!")
  ```

### `add-line` / `cad:add-line`
- **Syntax**: `(add-line pt1 pt2 [layer])`
- **Summary**: Directly creates a line entity between `pt1` and `pt2` on optional `layer` (defaults to `"0"`) and returns its `ENAME`.
- **Example**:
  ```lisp
  (add-line '(0 0 0) '(100 50 0) "WALLS")
  ```

### `add-circle` / `cad:add-circle`
- **Syntax**: `(add-circle center radius [layer])`
- **Summary**: Directly creates a circle entity at `center` with `radius` on optional `layer` and returns its `ENAME`.
- **Example**:
  ```lisp
  (add-circle '(50 50 0) 20.0 "CIRCLES")
  ```

---

## 10. Entity Manipulation (DXF)

### `entmake` / `entmakex`
- **Syntax**: `(entmake dxf-alist)` / `(entmakex dxf-alist)`
- **Summary**: Creates a new CAD entity (`"LINE"`, `"CIRCLE"`, `"POINT"`) from a DXF group-code association list (`0`=type, `8`=layer, `10`=start/center point, `11`=end point, `40`=radius, `62`=ACI color index). `entmake` returns the input list; `entmakex` returns the new `ENAME`.
- **Example**:
  ```lisp
  (entmake (list '(0 . "CIRCLE")
                 '(8 . "SHAPES")
                 (cons 10 '(0.0 0.0 0.0))
                 (cons 40 30.0)
                 (cons 62 1))) ; ACI 1 = Red
  ```

### `entget`
- **Syntax**: `(entget ename [app-list])`
- **Summary**: Retrieves the DXF group-code association list for entity `ename`.
- **Example**:
  ```lisp
  (setq data (entget (entlast)))
  (cdr (assoc 40 data)) ; => radius of last circle
  ```

### `entmod` / `entupd`
- **Syntax**: `(entmod dxf-alist)` / `(entupd ename)`
- **Summary**: Modifies an existing entity's properties using an updated DXF association list (`entmod`) and refreshes its display (`entupd`).
- **Example**:
  ```lisp
  (entmod (subst (cons 40 50.0) (assoc 40 data) data))
  ```

### `entdel`
- **Syntax**: `(entdel ename)`
- **Summary**: Deletes the specified entity `ename` from the drawing.
- **Example**:
  ```lisp
  (entdel (entlast))
  ```

### `entlast` / `entnext` / `handent`
- **Syntax**: `(entlast)`, `(entnext [ename])`, `(handent handle-str)`
- **Summary**: `entlast` returns the `ENAME` of the most recently created entity. `entnext` steps through database entities. `handent` looks up an entity by its hexadecimal handle string.
- **Example**:
  ```lisp
  (setq last-ent (entlast))
  ```

---

## 11. Selection Sets

### `ssget`
- **Syntax**: `(ssget [mode] [filter-alist])`
- **Summary**: Creates a selection set of entities matching optional `mode` (e.g., `"X"`, `"_X"`) and optional DXF `filter-alist`.
- **Example**:
  ```lisp
  (setq ss (ssget "X" '((0 . "CIRCLE"))))
  ```

### `sslength` / `ssname`
- **Syntax**: `(sslength ss)` / `(ssname ss index)`
- **Summary**: `sslength` returns the number of entities in selection set `ss`. `ssname` returns the `ENAME` at 0-based `index`.
- **Example**:
  ```lisp
  (setq n (sslength ss))
  (setq first-ent (ssname ss 0))
  ```

### `ssadd` / `ssdel` / `ssmemb`
- **Syntax**: `(ssadd [ename [ss]])`, `(ssdel ename ss)`, `(ssmemb ename ss)`
- **Summary**: Adds an entity to a selection set (`ssadd`), removes an entity (`ssdel`), or tests if an entity belongs to `ss` (`ssmemb`).
- **Example**:
  ```lisp
  (ssadd (entlast) ss)
  ```

---

## 12. Interactive User Prompts

### `getpoint` / `getcorner`
- **Syntax**: `(getpoint [base-pt] [prompt])` / `(getcorner base-pt [prompt])`
- **Summary**: Prompts the user to pick a point in the viewport. When `base-pt` is provided, draws a live rubber-band line (`getpoint`) or rectangle (`getcorner`) from `base-pt` to the cursor.
- **Example**:
  ```lisp
  (setq p1 (getpoint "\nPick first corner: "))
  (setq p2 (getcorner p1 "\nPick opposite corner: "))
  ```

### `getdist`
- **Syntax**: `(getdist [base-pt] [prompt])`
- **Summary**: Prompts the user to specify a distance by picking one or two points on screen with a live rubber-band line, returning the distance as a `Real`.
- **Example**:
  ```lisp
  (setq rad (getdist pt "\nSpecify circle radius: "))
  ```

### `getangle` / `getorient`
- **Syntax**: `(getangle [base-pt] [prompt])` / `(getorient [base-pt] [prompt])`
- **Summary**: Prompts the user for an angle and returns the value in radians.
- **Example**:
  ```lisp
  (setq ang (getangle pt "\nSpecify rotation angle: "))
  ```

### `getstring` / `getint` / `getreal` / `getkword` / `initget`
- **Syntax**:
  - `(getstring [cr-flag] [prompt])`
  - `(getint [prompt])`
  - `(getreal [prompt])`
  - `(getkword [prompt])`
  - `(initget [bits] [keywords])`
- **Summary**: Prompts for a string, integer, floating-point number, or keyword constrained by `initget`.
- **Example**:
  ```lisp
  (initget 1 "Yes No")
  (setq ans (getkword "\nContinue? [Yes/No]: "))
  ```

---

## 13. OpenCAD Native UI (`ocl-*`) — Dialogs & Dockable Panels

### Control Constructors
Used inside both modal dialogs (`ocl-dialog`) and dockable side panels (`ocl-panel-register`):

- **`ocl-text`**: `(ocl-text id label [default])`
  - Creates a single-line text input box.
  - Example: `(ocl-text "txt_name" "Name" "Part-A")`
- **`ocl-number`**: `(ocl-number id label [default] [opts])`
  - Creates a numeric input field with optional `'((min . 0) (max . 100) (step . 1))`.
  - Example: `(ocl-number "radius" "Radius" 25.0)`
- **`ocl-checkbox`**: `(ocl-checkbox id label [default-bool])`
  - Creates a boolean toggle checkbox (`T` or `nil`).
  - Example: `(ocl-checkbox "chk_x" "Draw 'X' Lines" nil)`
- **`ocl-choice`**: `(ocl-choice id label options-list [default])`
  - Creates a dropdown selector menu from a list of string options.
  - Example: `(ocl-choice "col" "Color" '("Red" "Blue") "Red")`
- **`ocl-color`**: `(ocl-color id label [default-rgb])`
  - Creates an RGB color picker (`'(r g b)` where values are `0..255`).
  - Example: `(ocl-color "clr" "Fill Color" '(255 0 0))`
- **`ocl-file`**: `(ocl-file id label [default-path] [opts])`
  - Creates a native file open/save browser control.
  - Example: `(ocl-file "path" "Output File" "out.dxf")`
- **`ocl-table`**: `(ocl-table id label headers [rows] [opts])`
  - Creates an editable multi-column table grid.
  - Example: `(ocl-table "tbl" "Points" '("X" "Y") '(("0" "0")))`
- **`ocl-fields`**: `(ocl-fields [id] title [opts] ctrl1 ...)`
  - Creates a collapsible group box containing nested controls.
  - Example: `(ocl-fields "Advanced" (ocl-checkbox "snap" "Snap" T))`
- **`ocl-button`**: `(ocl-button id label [action_or_opts] [opts])`
  - Creates a clickable button. In a panel, `action` is an AutoLISP command string executed on click.
  - Options alist supports `(align-h . "left"|"center"|"right")` and `(width . float)`.
  - Example: `(ocl-button "btn_draw" "Draw Shape" "(c:draw-shape)" '((align-h . "center") (width . 140.0)))`
- **`ocl-info`**: `(ocl-info text [opts])`
  - Creates a read-only static label / informational text.
  - Options alist supports `(align-h . "left"|"center"|"right")`.
  - Example: `(ocl-info "Select options below:" '((align-h . "center")))`
- **`ocl-row`**: `(ocl-row [opts] ctrl1 ctrl2 ...)`
  - Arranges multiple controls horizontally on a single row.
  - Options alist supports `(align-h . "left"|"center"|"right")` and `(align-v . "top"|"middle"|"bottom")`.
  - Example: `(ocl-row '((align-h . "center") (align-v . "middle")) (ocl-button "b1" "1" "(k 1)") (ocl-button "b2" "2" "(k 2)"))`
- **`ocl-column`**: `(ocl-column [opts] ctrl1 ctrl2 ...)`
  - Arranges multiple controls vertically in a single column container.
  - Options alist supports `(align-h . "left"|"center"|"right")` and `(align-v . "top"|"middle"|"bottom")`.
  - Example: `(ocl-column '((align-h . "center")) (ocl-text "t1" "Label" "") (ocl-button "b1" "OK"))`

### Modal Dialog Functions

#### `ocl-dialog` & `ocl-dialog-show`
- **Syntax**:
  - `(ocl-dialog title [options-alist] row1 [row2 ...])`
  - `(ocl-dialog-show dialog-def)`
- **Summary**: `ocl-dialog` builds a dialog definition (optional alist supports `'((width . 400.0) (submit . "OK") (cancel . "Cancel"))`). `ocl-dialog-show` displays it modally and returns an association list of `(id . value)` pairs when submitted, or `nil` if cancelled.
- **Example**:
  ```lisp
  (setq dlg (ocl-dialog "Circle Parameters"
              (ocl-number "rad" "Radius" 30.0)
              (ocl-checkbox "cross" "Add Crosshairs" T)))
  (setq res (ocl-dialog-show dlg))
  (if res
    (princ (ocl-get-real "rad" res))
  )
  ```

#### Dialog Result Getters (`ocl-get*`)
- **Syntax**:
  - `(ocl-get key alist)`
  - `(ocl-get-int key alist)`
  - `(ocl-get-real key alist)`
  - `(ocl-get-str key alist)`
  - `(ocl-get-bool key alist)`
- **Summary**: Extracts and coerces control values by `key` from the association list returned by `ocl-dialog-show`.
- **Example**:
  ```lisp
  (setq r (ocl-get-real "rad" res))
  ```

### Dockable Side Panel Functions

#### `ocl-panel-register`
- **Syntax**: `(ocl-panel-register panel-id title [options-alist] ctrl1 [ctrl2 ...])`
- **Summary**: Registers a persistent dockable panel in the OpenCAD Studio side pane and seeds all default control states.
- **Example**:
  ```lisp
  (ocl-panel-register "shape_panel" "Shape Generator"
    '((width . 260.0))
    (ocl-choice "choice_color" "Color" '("Red" "Green" "Blue") "Red")
    (ocl-checkbox "chk_draw_x" "Draw 'X' Lines" nil)
    (ocl-button "btn_draw" "Draw Shape" "(c:draw-shape)")
  )
  ```

#### `ocl-panel-toggle`
- **Syntax**: `(ocl-panel-toggle panel-id [show-bool])`
- **Summary**: Opens, closes, or toggles visibility of a registered dockable panel. Pass `T` to explicitly show, or omit `show-bool` to toggle.
- **Example**:
  ```lisp
  (ocl-panel-toggle "shape_panel" T)
  ```

#### `ocl-panel-update`
- **Syntax**: `(ocl-panel-update panel-id values-alist)`
- **Summary**: Programmatically updates one or more control values in a live dockable panel without re-registering it.
- **Example**:
  ```lisp
  (ocl-panel-update "calc" (list (cons "display" "123.45")))
  ```

#### `ocl-panel-get` / `ocl-panel-get-num` / `ocl-panel-get-real` / `ocl-panel-get-bool` / `ocl-panel-get-str`
- **Syntax**:
  - `(ocl-panel-get panel-id control-id)`
  - `(ocl-panel-get-num panel-id control-id)`
  - `(ocl-panel-get-real panel-id control-id)`
  - `(ocl-panel-get-bool panel-id control-id)`
  - `(ocl-panel-get-str panel-id control-id)`
- **Summary**: Reads the current live value of any control (`checkbox`, `choice`, `text`, `number`, `color`) inside a dockable panel. Automatically synchronized whenever the user interacts with the panel UI.
- **Example**:
  ```lisp
  (setq col    (ocl-panel-get "shape_panel" "choice_color"))
  (setq draw-x (ocl-panel-get-bool "shape_panel" "chk_draw_x"))
  (setq radius (ocl-panel-get-num "shape_panel" "rad"))
  ```

---

## 14. Visual LISP (`vl-*`) Extensions

### String Utilities

- **`vl-string-search`**: `(vl-string-search pattern str [start-pos])`
  - Returns 0-based index of `pattern` in `str`, or `nil`.
  - Example: `(vl-string-search "CAD" "OpenCAD") ; => 4`
- **`vl-string-subst`**: `(vl-string-subst new-str pattern str [start-pos])`
  - Replaces the first occurrence of `pattern` in `str` with `new-str`.
  - Example: `(vl-string-subst "2D" "3D" "3D Model") ; => "2D Model"`
- **`vl-string-trim`**: `(vl-string-trim char-set str)`
  - Strips characters in `char-set` from both ends of `str`.
  - Example: `(vl-string-trim " \t\n" "  hello  ") ; => "hello"`
- **`vl-string-left-trim` / `vl-string-right-trim`**: `(vl-string-left-trim char-set str)`
  - Strips characters in `char-set` from the start or end of `str`.
  - Example: `(vl-string-left-trim " " "  abc") ; => "abc"`
- **`vl-string-translate`**: `(vl-string-translate src-set dest-set str)`
  - Replaces characters in `str` matching `src-set` with corresponding characters in `dest-set`.
  - Example: `(vl-string-translate "abc" "123" "cab") ; => "312"`
- **`vl-string->list` / `vl-list->string`**: `(vl-string->list str)` / `(vl-list->string int-list)`
  - Converts between a string and a list of ASCII integer codes.
  - Example: `(vl-string->list "AB") ; => (65 66)`
- **`vl-princ-to-string` / `vl-prin1-to-string`**: `(vl-princ-to-string expr)`
  - Formats `expr` as an unquoted (`princ`) or quoted (`prin1`) string.
  - Example: `(vl-princ-to-string 123) ; => "123"`
- **`vl-symbol-name`**: `(vl-symbol-name sym)`
  - Returns the name of symbol `sym` as a string.
  - Example: `(vl-symbol-name 'my-sym) ; => "MY-SYM"`

### List & Functional Utilities

- **`vl-remove`**: `(vl-remove elem list)`
  - Removes all elements equal to `elem` from `list`.
  - Example: `(vl-remove 0 '(1 0 2 0 3)) ; => (1 2 3)`
- **`vl-remove-if` / `vl-remove-if-not`**: `(vl-remove-if pred-fn list)`
  - Removes (`vl-remove-if`) or keeps (`vl-remove-if-not`) elements satisfying `pred-fn`.
  - Example: `(vl-remove-if 'minusp '(-1 2 -3 4)) ; => (2 4)`
- **`vl-position`**: `(vl-position elem list)`
  - Returns the 0-based index of `elem` in `list`, or `nil`.
  - Example: `(vl-position "B" '("A" "B" "C")) ; => 1`
- **`vl-sort` / `vl-sort-i`**: `(vl-sort list compare-fn)`
  - Sorts `list` (`vl-sort`) or returns sorted 0-based indices (`vl-sort-i`) using `compare-fn`.
  - Example: `(vl-sort '(5 2 8 1) '<) ; => (1 2 5 8)`
- **`vl-list-length`**: `(vl-list-length list)`
  - Returns list length for proper lists, or `nil` for dotted lists.
  - Example: `(vl-list-length '(1 2 3)) ; => 3`
- **`vl-member-if` / `vl-member-if-not`**: `(vl-member-if pred-fn list)`
  - Returns the tail of `list` starting at the first element satisfying (or not satisfying) `pred-fn`.
  - Example: `(vl-member-if 'zerop '(5 0 3)) ; => (0 3)`
- **`vl-every` / `vl-some`**: `(vl-every pred-fn list1 [list2 ...])`
  - Tests whether all (`vl-every`) or any (`vl-some`) elements satisfy `pred-fn`.
  - Example: `(vl-every 'numberp '(1 2 3)) ; => T`

### File System & Error Handling

- **`vl-filename-base` / `vl-filename-directory` / `vl-filename-extension`**: `(vl-filename-base path)`
  - Extracts the base name, directory path, or file extension from `path`.
  - Example: `(vl-filename-base "C:/cad/part.dxf") ; => "part"`
- **`vl-file-copy` / `vl-file-delete` / `vl-file-rename`**: `(vl-file-copy src dst [append])`
  - Copies, deletes, or renames files on disk.
  - Example: `(vl-file-copy "a.txt" "b.txt")`
- **`vl-file-size` / `vl-file-systime`**: `(vl-file-size path)`
  - Returns file size in bytes or modification timestamp list.
  - Example: `(vl-file-size "routine.lsp")`
- **`vl-directory-files`**: `(vl-directory-files [dir [pattern [mode]]])`
  - Lists files or folders in `dir` (`mode`: `-1`=dirs only, `1`=files only, `0`=all).
  - Example: `(vl-directory-files "." "*.lsp" 1)`
- **`vl-catch-all-apply` / `vl-catch-all-error-p` / `vl-catch-all-error-message`**:
  - Safely invokes a function, checks if the result is an error, and extracts the error message.
  - Example:
    ```lisp
    (setq r (vl-catch-all-apply '/ '(10 0)))
    (if (vl-catch-all-error-p r)
      (princ (vl-catch-all-error-message r))
    )
    ```
- **`vl-load-com`**: `(vl-load-com)`
  - Compatibility initializer that returns `T`.

---

## 15. ActiveX / VLA (`vlax-*`) Extensions

### Object Properties & Methods

- **`vlax-ename->vla-object` / `vlax-vla-object->ename`**:
  - Converts between an `ENAME` and a `VLA-OBJECT`.
  - Example: `(setq obj (vlax-ename->vla-object (entlast)))`
- **`vlax-get-property` / `vlax-get`**: `(vlax-get-property obj prop)`
  - Reads property `prop` (e.g., `'Layer`, `'Radius`, `'Center`, `'StartPoint`, `'EndPoint`, `'Length`, `'Area`, `'Color`).
  - Example: `(vlax-get-property obj 'Radius)`
- **`vlax-put-property` / `vlax-put`**: `(vlax-put-property obj prop val)`
  - Updates property `prop` on `obj` to `val`.
  - Example: `(vlax-put-property obj 'Layer "WALLS")`
- **`vlax-invoke-method` / `vlax-invoke`**: `(vlax-invoke-method obj method [args ...])`
  - Invokes a method on `obj` (e.g., `'Delete`, `'Update`).
  - Example: `(vlax-invoke-method obj 'Delete)`
- **`vlax-property-available-p` / `vlax-method-applicable-p`**:
  - Checks whether a property or method is supported by `obj`.
  - Example: `(vlax-property-available-p obj 'Radius)`

### Persistent LData Dictionary Storage

- **`vlax-ldata-put`**: `(vlax-ldata-put dict key val)`
  - Stores a Lisp value `val` under `(dict, key)`.
  - Example: `(vlax-ldata-put "MyApp" "Scale" 2.5)`
- **`vlax-ldata-get`**: `(vlax-ldata-get dict key [default])`
  - Retrieves the value stored at `(dict, key)`, or `default`.
  - Example: `(vlax-ldata-get "MyApp" "Scale" 1.0)`
- **`vlax-ldata-delete` / `vlax-ldata-list`**:
  - Deletes a key or lists all `(key . val)` entries in `dict`.
  - Example: `(vlax-ldata-list "MyApp")`

### Variants, SafeArrays & Curve Queries

- **`vlax-make-variant` / `vlax-variant-value`**:
  - Wraps and unwraps variant containers.
  - Example: `(vlax-variant-value (vlax-make-variant 10.0))`
- **`vlax-make-safearray` / `vlax-safearray-value`**:
  - Creates a SafeArray container and extracts its elements as a list.
  - Example: `(vlax-make-safearray 5 '(0 . 2))`
- **`vlax-curve-getstartpoint` / `vlax-curve-getendpoint`**: `(vlax-curve-getstartpoint curve)`
  - Returns the start or end point `(x y z)` of a curve entity.
  - Example: `(vlax-curve-getstartpoint (entlast))`
- **`vlax-curve-getdistatparam` / `vlax-curve-getparamatdist`**:
  - Converts between curve parameter and distance along the curve.
  - Example: `(vlax-curve-getdistatparam (entlast) 1.0)`
