(use gauche.test)
(use vize.process)

(test-start "Vize Gauche process runtime")

(test* "direct argv preserves spaces and newlines"
       "hello world\n"
       (capture! "printf" "%s\n" "hello world"))

(test* "successful commands return zero"
       0
       (run/exit-code "true"))

(test* "expected nonzero statuses remain observable"
       1
       (run/exit-code "false"))

(test* "required commands reject nonzero exits"
       (test-error)
       (run! "false"))

(test-end)
