(require 'ert)
(require 'eglot)

(load-file (expand-file-name "../vize.el" (file-name-directory load-file-name)))

(defconst vize-test--off
  '(:enabled :json-false :nativeLinkedEditing :json-false :diagnostics :json-false
    :lint :json-false :typecheck :json-false :editor :json-false
    :ecosystem :json-false :optionsApi :json-false :legacyVue2 :json-false
    :completion :json-false :signatureHelp :json-false :hover :json-false
    :definition :json-false :references :json-false :documentSymbols :json-false
    :workspaceSymbols :json-false :codeActions :json-false :rename :json-false
    :formatting :json-false :codeLens :json-false :semanticTokens :json-false
    :documentLinks :json-false :foldingRanges :json-false :inlayHints :json-false
    :fileRename :json-false :autoInsert :json-false :corsa :json-false
    :tsgo :json-false :crossFile :json-false))

(defun vize-test--lint ()
  (let ((options (copy-tree vize-test--off)))
    (setq options (plist-put options :enabled t))
    (plist-put options :lint t)))

(ert-deftest vize-eglot-default-program ()
  (should
   (equal (vize-eglot-server-program) '("vize" "lsp"))))

(ert-deftest vize-eglot-recommended-program ()
  (should
   (equal (vize-eglot-server-program 'recommended)
          '("vize" "lsp" :initializationOptions
            (:editor t :ecosystem t :lint t :typecheck t)))))

(ert-deftest vize-eglot-off-program ()
  (should (equal (vize-eglot-server-program 'off)
                 (list "vize" "lsp" :initializationOptions vize-test--off))))

(ert-deftest vize-profile-options-lint ()
  (should (equal (vize-profile-options 'lint) (vize-test--lint))))

(ert-deftest vize-profile-options-uses-default-profile-variable ()
  (let ((vize-eglot-profile 'lint))
    (should (equal (vize-profile-options) (vize-test--lint)))))

(ert-deftest vize-profile-options-off ()
  (should (equal (vize-profile-options 'off) vize-test--off)))

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
            (list "/tmp/vize" "lsp" "--debug" :initializationOptions (vize-test--lint))))))

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
            (list (list 'vize-art-vue-mode "vize" "lsp" :initializationOptions (vize-test--lint))
                  (list 'vize-vue-mode "vize" "lsp" :initializationOptions (vize-test--lint)))))))

(ert-deftest vize-setup-eglot-off-profile-registers-explicit-disabled-options ()
  (let ((eglot-server-programs nil)
        (vize-eglot-major-modes '(vize-vue-mode)))
    (vize-setup-eglot 'off)
    (should
     (equal eglot-server-programs
            (list (list 'vize-vue-mode "vize" "lsp" :initializationOptions vize-test--off))))))

(ert-deftest vize-setup-eglot-default-defers-project-settings ()
  (let ((eglot-server-programs nil)
        (vize-eglot-major-modes '(vize-vue-mode)))
    (vize-setup-eglot)
    (should (equal eglot-server-programs '((vize-vue-mode "vize" "lsp"))))))

(ert-deftest vize-profiles-preserve-real-json-false-for-every-explicit-override ()
  (dolist (profile '(off lint))
    (let* ((wire (json-parse-string (jsonrpc--json-encode (vize-profile-options profile))
                                   :object-type 'plist :false-object :json-false))
           (expected (if (eq profile 'off) vize-test--off (vize-test--lint))))
      (dolist (key (cl-loop for (key value) on vize-test--off by #'cddr collect key))
        (should (plist-member wire key))
        (should (eq (plist-get wire key) (plist-get expected key)))))))

(ert-deftest vize-default-and-partial-options-preserve-project-inheritance ()
  (let ((project '(:hover t :formatting t :codeActions t :crossFile t)))
    (should-not (vize-profile-options))
    (should (equal (vize-eglot-server-program) '("vize" "lsp")))
    ;; Eglot accepts custom partial initialization options directly; decode
    ;; its actual wire representation before applying the named override.
    (let* ((partial (json-parse-string (jsonrpc--json-encode '(:hover :json-false))
                                      :object-type 'plist :false-object :json-false))
           (merged (copy-tree project)))
      (cl-loop for (key value) on partial by #'cddr
               do (setq merged (plist-put merged key value)))
      (should (equal merged '(:hover :json-false :formatting t :codeActions t :crossFile t))))))

(when (getenv "VIZE_PROFILE_PACKET")
  (write-region
   (jsonrpc--json-encode
    (list :defaults (vize-profile-options)
          :partial '(:hover :json-false)
          :recommended (vize-profile-options 'recommended)
          :lint (vize-profile-options 'lint)
          :off (vize-profile-options 'off)))
   nil (getenv "VIZE_PROFILE_PACKET") nil 'silent))
