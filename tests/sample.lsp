; Sample AutoLISP script demonstrating calculations, strings, and Visual LISP
(setq title "OpenCAD LISP Engine Test"
      radius 50.0
      area (* pi radius radius))

(princ (strcat "\nTitle: " title "\n"))
(princ (strcat "Area of circle with radius " (rtos radius 2 2) " is " (rtos area 2 2) "\n"))

; Test list manipulation and sorting
(setq numbers '(42 7 19 3 88 1))
(setq sorted-numbers (vl-sort numbers '<))
(princ (strcat "Sorted numbers: " (vl-string-trim "()" (vl-prin1-to-string sorted-numbers)) "\n"))

; Test user-defined function with local variables
(defun hypotenuse (a b / sum_sq)
  (setq sum_sq (+ (* a a) (* b b)))
  (sqrt sum_sq)
)

(princ (strcat "Hypotenuse of 3 and 4: " (rtos (hypotenuse 3 4) 2 2) "\n"))
