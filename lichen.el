(setq lkeywords '("if" "or" "next" "await" "emit" "with"))
(setq lkeywordsr (regexp-opt lkeywords 'words))

(setq font-lock-keywords
      `(
        (, "#.+" . font-lock-comment-face)
        (, "\:\\w+" . font-lock-constant-face)
        (, "^\\w+.+" . font-lock-function-name-face)

        ;(,"`[A-Za-z0-9_|\.|\-]+" . font-lock-variable-name-face)
        
        (,  "\".+\"" . font-lock-string-face)

        (, "['|!]" . font-lock-negation-char-face)
        (, ".[0-9]+" . font-lock-type-face)
        (, "[<|>|\+]+" . font-lock-type-face)
        (, "(.+)+" . font-lock-type-face)

        (, "[\{|\}|\[]+" . font-lock-keyword-face)
        (, "]+" . font-lock-keyword-face)
        

        ;(, "\\(next.\\).+." . font-lock-variable-name-face)
        (, lkeywordsr . font-lock-keyword-face)
        
        ;(, "\\s\"" . font-lock-string-delimiter-face)
        ))


(defun lichen-mode ()
  "Major mode for editing lichen DSL"
  (interactive)
  (kill-all-local-variables)
  
  (set-syntax-table text-mode-syntax-table)
  (set (make-local-variable 'font-lock-defaults) '(font-lock-keywords))
  (setq major-mode 'lichen-mode)
  (setq mode-name "lichen")
  (run-hooks 'lichen-mode-hook))


(add-to-list 'auto-mode-alist '("\\.ls\\'" . lichen-mode))
(provide 'lichen-mode)
