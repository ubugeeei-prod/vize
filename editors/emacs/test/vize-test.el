(require 'ert)

(load-file (expand-file-name "../vize.el" (file-name-directory load-file-name)))

(ert-deftest vize-eglot-default-program ()
  (should
   (equal (vize-eglot-server-program)
          '("vize" "lsp" :initializationOptions
            (:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-recommended-program ()
  (should
   (equal (vize-eglot-server-program 'recommended)
          '("vize" "lsp" :initializationOptions
            (:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-off-program ()
  (should (equal (vize-eglot-server-program 'off) '("vize" "lsp"))))

(ert-deftest vize-profile-options-lint ()
  (should (equal (vize-profile-options 'lint) '(:lint t))))

(ert-deftest vize-profile-options-off ()
  (should (equal (vize-profile-options 'off) nil)))

(ert-deftest vize-profile-options-rejects-unknown-profile ()
  (should-error (vize-profile-options 'missing) :type 'error))

(ert-deftest vize-profile-options-returns-copy ()
  (let ((options (vize-profile-options 'recommended)))
    (setf (plist-get options :lint) nil)
    (should
     (equal (vize-profile-options 'recommended)
            '(:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-custom-command ()
  (let ((vize-eglot-command '("/tmp/vize" "lsp" "--debug")))
    (should
     (equal (vize-eglot-server-program 'lint)
            '("/tmp/vize" "lsp" "--debug" :initializationOptions (:lint t))))))

(ert-deftest vize-eglot-server-program-copies-command ()
  (let* ((vize-eglot-command '("/tmp/vize" "lsp"))
         (program (vize-eglot-server-program 'off)))
    (setcar program "changed")
    (should (equal vize-eglot-command '("/tmp/vize" "lsp")))))
