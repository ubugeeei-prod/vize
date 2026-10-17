;;; emacs-native-config.el --- Real Eglot project settings -*- lexical-binding: t; -*-

(require 'ert)
(require 'eglot)
(require 'json)
(require 'seq)

(load-file (expand-file-name "editors/emacs/vize.el" (getenv "VIZE_EMACS_REPOSITORY")))

(defvar vize-real--cases nil)

(defun vize-real--entries ()
  "Read complete packets retained by the transparent real-server tap."
  (let ((file (getenv "VIZE_CONFORMANCE_TRANSCRIPT")) entries)
    (when (file-exists-p file)
      (with-temp-buffer
        (insert-file-contents file)
        (dolist (line (split-string (buffer-string) "\n" t))
          (condition-case nil
              (push (json-parse-string line :object-type 'plist :array-type 'array
                                       :null-object nil :false-object :json-false)
                    entries)
            (json-parse-error nil)))))
    (nreverse entries)))

(defun vize-real--wait (label predicate)
  "Allow the real client to process input until PREDICATE satisfies LABEL."
  (let ((deadline (+ (float-time) 30)) found)
    (while (and (not (setq found (funcall predicate))) (< (float-time) deadline))
      ;; File notifications are editor events, including in a batch ERT host.
      ;; GNU Emacs's own filenotify tests pump them with `read-event'.
      (read-event nil nil 0.05))
    (unless found (ert-fail (format "Timed out waiting for %s" label)))
    found))

(defun vize-real--open (name)
  "Open NAME through the packaged Vue mode and ordinary Eglot startup."
  (let ((buffer (find-file-noselect
                 (expand-file-name name (getenv "VIZE_EMACS_WORKSPACE")))))
    (with-current-buffer buffer
      (should (eq major-mode 'vize-vue-mode))
      (eglot-ensure)
      ;; `eglot-ensure' starts after the next editor command, also in batch.
      (run-hooks 'post-command-hook)
      (vize-real--wait "Eglot management" #'eglot-managed-p))
    buffer))

(defun vize-real--diagnostics (buffer deprecated &optional version)
  "Wait for BUFFER's complete native publication and DEPRECATED rule state."
  (let ((uri (with-current-buffer buffer
               (plist-get (eglot--TextDocumentIdentifier) :uri))))
    (vize-real--wait
     (format "%s diagnostics (deprecated=%s)" (buffer-name buffer) deprecated)
     (lambda ()
       (let* ((entry
               (seq-find
                (lambda (entry)
                  (let ((message (plist-get entry :msg)))
                    (and (equal (plist-get entry :dir) "s2c")
                         (equal (plist-get message :method) "textDocument/publishDiagnostics")
                         (equal (plist-get (plist-get message :params) :uri) uri))))
                (reverse (vize-real--entries))))
              (params (plist-get (plist-get entry :msg) :params))
              (matched
               (seq-some (lambda (diagnostic)
                           (equal (plist-get diagnostic :code)
                                  "vue/no-deprecated-v-bind-sync"))
                         (plist-get params :diagnostics))))
         (and params (>= (or (plist-get params :version) 0) (or version 0))
              (eq (not (null matched)) deprecated)
              params))))))

(defun vize-real--hover (buffer)
  "Request native hover through the real Eglot JSON-RPC connection."
  (with-current-buffer buffer
    (jsonrpc-request (eglot-current-server) :textDocument/hover
                     `(:textDocument ,(eglot--TextDocumentIdentifier)
                       :position (:line 1 :character 8))
                     :timeout 15)))

(defun vize-real--watch-ready (server)
  "Require actual Eglot capability advertisement and native watch installation."
  (let* ((initialize
          (seq-find (lambda (entry)
                      (equal (plist-get (plist-get entry :msg) :method) "initialize"))
                    (vize-real--entries)))
         (capabilities (plist-get (plist-get (plist-get initialize :msg) :params)
                                  :capabilities)))
    (should (eq (plist-get (plist-get (plist-get capabilities :workspace)
                                     :didChangeWatchedFiles)
                          :dynamicRegistration) t)))
  (vize-real--wait "native watcher registration in the actual Eglot host"
                   (lambda () (> (hash-table-count (eglot--file-watches server)) 0))))

(defun vize-real--change-config (name source)
  "Save configuration and wait for Eglot's own file-notify notification."
  (let* ((file (expand-file-name name (getenv "VIZE_EMACS_WORKSPACE")))
         (uri (eglot--path-to-uri file))
         (before (length (vize-real--entries))))
    (with-temp-file file (insert source))
    (vize-real--wait
     (format "Eglot file notification for %s" name)
     (lambda ()
       (seq-some
        (lambda (entry)
          (let ((message (plist-get entry :msg)))
            (and (equal (plist-get entry :dir) "c2s")
                 (equal (plist-get message :method) "workspace/didChangeWatchedFiles")
                 (seq-some (lambda (change) (equal (plist-get change :uri) uri))
                           (plist-get (plist-get message :params) :changes)))))
        (nthcdr before (vize-real--entries)))))
    (list :file name :source source)))

(defun vize-real--touch (buffer)
  "Make a real saved editor change and return its Eglot document version."
  (with-current-buffer buffer
    (goto-char (point-max))
    (insert "\n")
    (save-buffer)
    eglot--versioned-identifier))

(defun vize-real--case (name legacy modern deprecated hover-enabled &optional config)
  "Observe fresh complete native packets after ordinary editor saves."
  (let* ((legacy-version (vize-real--touch legacy))
         (modern-version (vize-real--touch modern))
         (legacy-packet (vize-real--diagnostics legacy deprecated legacy-version))
         (modern-packet (vize-real--diagnostics modern t modern-version))
         (hover (vize-real--wait
                 (format "%s native hover feature state" name)
                 (lambda ()
                   (condition-case nil
                       (let ((value (vize-real--hover modern)))
                         (when (eq (not (null value)) hover-enabled)
                           (list value)))
                     ;; A native owner can retire while a watched reload is pending.
                     ;; Preserve that wire error, then require the final feature state.
                     (jsonrpc-error nil))))))
    (vize-real--record name legacy-packet modern-packet (car hover) config)))

(defun vize-real--record (name legacy modern hover &optional config)
  "Keep complete case observations in addition to the unmodified wire packets."
  (push (list :phase name :legacy legacy :modern modern :hover hover
              :authoredConfig config :wireCount (length (vize-real--entries)))
        vize-real--cases)
  (with-temp-file (getenv "VIZE_EMACS_CASES")
    (insert (json-serialize (vconcat (reverse vize-real--cases))
                            :null-object nil :false-object :json-false))))

(ert-deftest vize-eglot-real-vite-and-reference-ownership ()
  (let ((eglot-sync-connect t)
        (eglot-connect-timeout 60)
        (eglot-autoshutdown t)
        (eglot-send-changes-idle-time 0.05)
        (vize-eglot-profile nil)
        legacy modern server)
    (unwind-protect
        (progn
          (vize-setup-eglot)
          (setq legacy (vize-real--open "Legacy.vue"))
          (setq server (with-current-buffer legacy (eglot-current-server)))
          (setq modern (vize-real--open "Modern.vue"))
          (should (eq server (with-current-buffer modern (eglot-current-server))))
          (vize-real--watch-ready server)
          (let ((legacy-packet (vize-real--diagnostics legacy nil))
                (modern-packet (vize-real--diagnostics modern t))
                (hover (vize-real--hover modern)))
            (should-not hover)
            (vize-real--record "initial-reference-targets-and-vite-hover"
                               legacy-packet modern-packet hover))
          (let* ((vite-file (expand-file-name "vue-policy.mjs" (getenv "VIZE_EMACS_WORKSPACE")))
                 (legacy-file (expand-file-name "tsconfig.legacy.json" (getenv "VIZE_EMACS_WORKSPACE")))
                 (vite-source (with-temp-buffer (insert-file-contents vite-file) (buffer-string)))
                 (legacy-source (with-temp-buffer (insert-file-contents legacy-file) (buffer-string)))
                 (enabled-source (replace-regexp-in-string "\"hover\":false" "\"hover\":true" vite-source)))
            (vize-real--case "vite-valid-hover-enabled" legacy modern nil t
                              (vize-real--change-config "vue-policy.mjs" enabled-source))
            (vize-real--case "vite-invalid-retains-hover-enabled" legacy modern nil t
                              (vize-real--change-config "vue-policy.mjs" "export default {"))
            (vize-real--case "vite-valid-hover-recovery" legacy modern nil nil
                              (vize-real--change-config "vue-policy.mjs" vite-source))
            (vize-real--case "reference-valid-target-3" legacy modern t nil
                              (vize-real--change-config
                               "tsconfig.legacy.json"
                               (replace-regexp-in-string "2\\.7" "3" legacy-source)))
            (vize-real--case "reference-invalid-retains-target-3" legacy modern t nil
                              (vize-real--change-config "tsconfig.legacy.json" "{\"vueCompilerOptions\":"))
            (vize-real--case "reference-valid-target-recovery" legacy modern nil nil
                              (vize-real--change-config "tsconfig.legacy.json" legacy-source))))
      (when server (ignore-errors (eglot-shutdown server nil 10)))
      (dolist (buffer (list legacy modern))
        (when (buffer-live-p buffer)
          (with-current-buffer buffer (set-buffer-modified-p nil))
          (kill-buffer buffer))))))

;;; emacs-native-config.el ends here
