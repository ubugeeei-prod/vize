(require 'ert)

(load-file (expand-file-name "../vize.el" (file-name-directory load-file-name)))

(ert-deftest vize-eglot-default-program ()
  (should
   (equal (vize-eglot-server-program)
          '("vize" "lsp" :initializationOptions (:lint t)))))

(ert-deftest vize-eglot-recommended-program ()
  (should
   (equal (vize-eglot-server-program 'recommended)
          '("vize" "lsp" :initializationOptions
            (:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-off-program ()
  (should (equal (vize-eglot-server-program 'off) '("vize" "lsp"))))

(ert-deftest vize-eglot-custom-command ()
  (let ((vize-eglot-command '("/tmp/vize" "lsp" "--debug")))
    (should
     (equal (vize-eglot-server-program 'lint)
            '("/tmp/vize" "lsp" "--debug" :initializationOptions (:lint t))))))
