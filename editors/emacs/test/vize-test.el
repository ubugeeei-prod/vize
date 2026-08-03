(require 'ert)
(require 'eglot)

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

(ert-deftest vize-profile-options-uses-default-profile-variable ()
  (let ((vize-eglot-profile 'lint))
    (should (equal (vize-profile-options) '(:lint t)))))

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

(ert-deftest vize-eglot-server-program-copies-initialization-options ()
  (let* ((program (vize-eglot-server-program 'recommended))
         (options (cadr (memq :initializationOptions program))))
    (plist-put options :lint nil)
    (should
     (equal (vize-profile-options 'recommended)
            '(:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-server-program-copies-command ()
  (let* ((vize-eglot-command '("/tmp/vize" "lsp"))
         (program (vize-eglot-server-program 'off)))
    (setcar program "changed")
    (should (equal vize-eglot-command '("/tmp/vize" "lsp")))))

(ert-deftest vize-eglot-server-program-does-not-mutate-command-for-profile-options ()
  (let* ((vize-eglot-command '("/tmp/vize" "lsp"))
         (program (vize-eglot-server-program 'lint)))
    (setcar program "changed")
    (should (equal vize-eglot-command '("/tmp/vize" "lsp")))))

(ert-deftest vize-profile-options-recommended-copy-is-independent ()
  (let ((first-copy (vize-profile-options 'recommended))
        (second-copy (vize-profile-options 'recommended)))
    (plist-put first-copy :editor nil)
    (should (equal second-copy '(:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-major-modes-cover-common-and-fallback-modes ()
  (dolist (mode '(vue-mode vue-ts-mode web-mode vize-vue-mode vize-art-vue-mode))
    (should (memq mode vize-eglot-major-modes))))

(ert-deftest vize-auto-mode-alist-registers-vue-patterns ()
  (should (equal (cdr (assoc "\\.vue\\'" auto-mode-alist)) 'vize-vue-mode))
  (should (equal (cdr (assoc "\\.art\\.vue\\'" auto-mode-alist)) 'vize-art-vue-mode)))

(ert-deftest vize-setup-eglot-registers-each-configured-major-mode ()
  (let ((eglot-server-programs nil)
        (vize-eglot-major-modes '(vize-vue-mode vize-art-vue-mode)))
    (vize-setup-eglot 'lint)
    (should
     (equal eglot-server-programs
            '((vize-art-vue-mode "vize" "lsp" :initializationOptions (:lint t))
              (vize-vue-mode "vize" "lsp" :initializationOptions (:lint t)))))))

(ert-deftest vize-setup-eglot-off-profile-registers-command-only ()
  (let ((eglot-server-programs nil)
        (vize-eglot-major-modes '(vize-vue-mode)))
    (vize-setup-eglot 'off)
    (should (equal eglot-server-programs '((vize-vue-mode "vize" "lsp"))))))
