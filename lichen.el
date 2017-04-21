(setq lkeywords '("if" "next" "await" "emit"))
(setq lkeywordsr (regexp-opt lkeywords 'words))

(setq font-lock-keywords
      `(
        (, "#.+" . font-lock-comment-face)
        (, "^\\w+.+" . font-lock-function-name-face)
        (,  "\".+\"" . font-lock-string-face)

        (, "'" . font-lock-string-constant-face)
        (, "\\s\"" . font-lock-string-delimiter-face)

        (, lkeywordsr . font-lock-keyword-face)
        ))

(define-derived-mode lichen-mode c-mode "lichen"
  "Major mode for editing lichen DSL"
  (setq font-lock-defaults '((font-lock-keywords))))


(add-to-list 'auto-mode-alist '("\\.ls\\'" . lichen-mode))
(provide 'lichen-mode)
