;;; vize.el --- Eglot integration for Vize -*- lexical-binding: t; -*-

;; Copyright (C) Vize contributors
;; SPDX-License-Identifier: MIT

;;; Commentary:

;; Configure the Vize language server for Eglot.

;;; Code:

(defgroup vize nil
  "Vize language server integration."
  :group 'tools
  :prefix "vize-")

(defcustom vize-eglot-command '("vize" "lsp")
  "Command used to start the Vize language server."
  :type '(repeat string)
  :group 'vize)

(defcustom vize-eglot-profile 'lint
  "Default Vize feature profile for Eglot."
  :type '(choice (const :tag "Lint only" lint)
                 (const :tag "Recommended" recommended)
                 (const :tag "Off" off))
  :group 'vize)

(defcustom vize-eglot-major-modes
  '(vue-mode vue-ts-mode web-mode vize-vue-mode vize-art-vue-mode)
  "Major modes that should start the Vize language server."
  :type '(repeat symbol)
  :group 'vize)

(defconst vize--profiles
  '((lint . (:lint t))
    (off . nil)
    (recommended . (:editor t :ecosystem t :lint t :typecheck t)))
  "Vize initialization option profiles.")

;;;###autoload
(define-derived-mode vize-vue-mode prog-mode "Vue"
  "Fallback major mode for Vue single-file components.")

;;;###autoload
(define-derived-mode vize-art-vue-mode prog-mode "Art Vue"
  "Fallback major mode for Art Vue single-file components.")

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.vue\\'" . vize-vue-mode))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.art\\.vue\\'" . vize-art-vue-mode))

(defun vize-profile-options (&optional profile)
  "Return initialization options for PROFILE."
  (let* ((profile (or profile vize-eglot-profile))
         (entry (assq profile vize--profiles)))
    (unless entry
      (error "Unknown Vize profile: %S" profile))
    (copy-tree (cdr entry))))

(defun vize-eglot-server-program (&optional profile)
  "Return an `eglot-server-programs' value for PROFILE."
  (let ((command (copy-sequence vize-eglot-command))
        (options (vize-profile-options profile)))
    (if options
        (append command (list :initializationOptions options))
      command)))

;;;###autoload
(defun vize-setup-eglot (&optional profile)
  "Register Vize with Eglot for PROFILE."
  (interactive)
  (with-eval-after-load 'eglot
    (dolist (mode vize-eglot-major-modes)
      (add-to-list 'eglot-server-programs
                   (cons mode (vize-eglot-server-program profile))))))

(provide 'vize)

;;; vize.el ends here
