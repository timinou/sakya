;;; prd-tasks.el --- PRD Task Management Validation and Automation -*- lexical-binding: t; -*-

;; Author: Design System Team
;; Version: 2.0.0
;; Package-Requires: ((emacs "29.1") (org "9.6"))
;; Keywords: org-mode, project-management, validation

;;; Commentary:

;; This module provides validation, automation, and dashboard functionality
;; for the PRD @tasks system.  It validates org-mode files containing
;; categories (PROJ-XXX, BUG-XXX, IMP-XXX) and tasks (ITEM-XXX), ensuring
;; they have required properties, valid agent references, and no circular
;; dependencies.
;;
;; Main entry points:
;;   - `prd-validate-file' - Validate a single @tasks file
;;   - `prd-validate-all' - Validate all @tasks files
;;   - `prd-dashboard' - Generate metrics dashboard
;;   - `prd-sync-backlinks' - Sync bidirectional links
;;   - `prd-audit-links' - Find broken links
;;
;; For Claude Code integration, use the -cli variants which output JSON:
;;   - `prd-validate-file-cli'
;;   - `prd-validate-all-cli'
;;   - `prd-dashboard-cli'

;;; Code:

(require 'org)
(require 'org-element)
(require 'json)
(require 'cl-lib)
(require 'seq)

;;; Customization

(defgroup prd-tasks nil
  "PRD Task Management System."
  :group 'org
  :prefix "prd-")

(defcustom prd-tasks-directory nil
  "Directory containing @tasks files.
If nil, automatically detected from current file."
  :type '(choice (const nil) directory)
  :group 'prd-tasks)

(defcustom prd-required-item-properties
  '("CUSTOM_ID" "AGENT" "EFFORT" "PRIORITY")
  "Properties required for every ITEM."
  :type '(repeat string)
  :group 'prd-tasks)

(defcustom prd-effort-regexp
  "^[0-9]+[hm]$"
  "Regular expression for valid EFFORT format (e.g., 1h, 30m)."
  :type 'regexp
  :group 'prd-tasks)

(defcustom prd-item-id-regexp
  "^ITEM-[0-9]+"
  "Regular expression for valid ITEM identifiers."
  :type 'regexp
  :group 'prd-tasks)

(defcustom prd-checkpoint-id-regexp
  "^CHK-[0-9]\\{3,\\}-[0-9]\\{2,\\}\\(-[a-z0-9-]*\\)?$"
  "Regular expression for valid CHK identifiers.
Format: CHK-NNN-MM or CHK-NNN-MM-slug where NNN is the parent
category number (3+ digits) and MM is the sequential checkpoint
number (2+ digits)."
  :type 'regexp
  :group 'prd-tasks)

(defcustom prd-checkpoint-heading-regexp
  "^\\(?:CHK-[0-9]+-[0-9]+\\(?:-[a-z0-9-]*\\)?\\|CHECKPOINT\\) "
  "Regular expression matching CHK heading titles.
Matches `CHK-NNN-MM description', `CHK-NNN-MM-slug description',
and `CHECKPOINT description' formats."
  :type 'regexp
  :group 'prd-tasks)

(defcustom prd-required-checkpoint-properties
  '("CUSTOM_ID" "CRITERIA" "VERIFY")
  "Properties required for every CHECKPOINT heading."
  :type '(repeat string)
  :group 'prd-tasks)

(defcustom prd-category-prefixes
  '("PROJ" "BUG" "IMP")
  "Prefixes for category identifiers (e.g., PROJ-001, BUG-001, IMP-001)."
  :type '(repeat string)
  :group 'prd-tasks)

(defcustom prd-category-subdirs
  '("projects" "bugfixes" "improvements")
  "Subdirectories under @tasks that contain task files."
  :type '(repeat string)
  :group 'prd-tasks)

(defcustom prd-todo-keywords
  '("ITEM" "DOING" "REVIEW" "DONE" "BLOCKED")
  "TODO keywords used in @tasks files."
  :type '(repeat string)
  :group 'prd-tasks)

(defun prd--category-id-regexp ()
  "Build regexp matching any category prefix (e.g., PROJ-001, BUG-001, IMP-001)."
  (concat "^\\(" (mapconcat #'regexp-quote prd-category-prefixes "\\|") "\\)-[0-9]+"))

;;; Internal Data Structures

(cl-defstruct (prd-validation-error
               (:constructor prd-make-error))
  "A validation error or warning."
  file line rule severity message hint context)

(cl-defstruct (prd-item
               (:constructor prd-make-item))
  "An ITEM extracted from an org file."
  id file line title status agent effort priority depends blocks properties closed-time level)

(cl-defstruct (prd-category
               (:constructor prd-make-category))
  "A category (PROJ/BUG/IMP) extracted from an org file."
  id file line title status goal depends-category items)

(cl-defstruct (prd-checkpoint
               (:constructor prd-make-checkpoint))
  "A checkpoint (CHK) grouping within a category."
  id file line title parent-category criteria verify review-by items level)

(cl-defstruct (prd-metrics
               (:constructor prd-make-metrics))
  "Metrics summary."
  total-items complete in-progress blocked pending)

;;; Directory Management

(defun prd--find-tasks-directory ()
  "Find the @tasks directory."
  (or prd-tasks-directory
      (when-let ((current (or buffer-file-name default-directory)))
        (when-let ((parent (locate-dominating-file current "@tasks")))
          (expand-file-name "@tasks" parent)))
      (let ((git-root (locate-dominating-file default-directory ".git")))
        (when git-root
          (expand-file-name "docs/@tasks" git-root)))))

(defun prd--tasks-directory ()
  "Return the @tasks directory path, ensuring it exists."
  (let ((dir (prd--find-tasks-directory)))
    (if (and dir (file-directory-p dir))
        (file-name-as-directory dir)
      (error "Cannot find @tasks directory"))))

(defun prd--all-org-files ()
  "Return list of all org files in @tasks directory tree."
  (let ((dir (prd--tasks-directory)))
    (directory-files-recursively dir "\\.org$")))

(defun prd--task-org-files ()
  "Return list of org files in task subdirectories only.
Scans `prd-category-subdirs' (projects/, bugfixes/, improvements/).
Use this for validation and metrics to avoid scanning documentation files."
  (let ((base (prd--tasks-directory))
        (files '()))
    (dolist (subdir prd-category-subdirs)
      (let ((dir (expand-file-name subdir base)))
        (when (file-directory-p dir)
          (setq files (append files (directory-files dir t "\\.org$"))))))
    ;; Filter out .gitkeep and similar non-org files
    (seq-filter (lambda (f) (string-suffix-p ".org" f)) files)))

(defun prd--agent-files ()
  "Return list of agent definition files.
Excludes index.org which is the registry document, not an agent."
  (let ((agents-dir (expand-file-name "agents" (prd--tasks-directory))))
    (when (file-directory-p agents-dir)
      (seq-filter (lambda (f)
                    (not (string= (file-name-nondirectory f) "index.org")))
                  (directory-files agents-dir t "\\.org$")))))

(defun prd--category-files ()
  "Return list of all category files across task subdirectories."
  (prd--task-org-files))

;;; Org Element Extraction

(defun prd--get-property (element property)
  "Get PROPERTY value from org ELEMENT."
  (org-element-property (intern (concat ":" property)) element))

(defun prd--extract-property (element property)
  "Extract PROPERTY from org ELEMENT (headline).
Handles both direct properties and those in property drawers."
  ;; First try direct property access (for properties parsed by org-element)
  (or (org-element-property (intern (concat ":" property)) element)
      ;; Then try property drawer
      (when-let ((drawer (prd--find-properties-drawer element)))
        (prd--property-from-drawer drawer property))))

(defun prd--extract-custom-id (element)
  "Extract CUSTOM_ID property from ELEMENT."
  (prd--extract-property element "CUSTOM_ID"))

(defun prd--find-properties-drawer (headline)
  "Find properties drawer in HEADLINE contents."
  (let ((contents (org-element-contents headline)))
    (seq-find (lambda (el)
                (eq (org-element-type el) 'property-drawer))
              contents)))

(defun prd--property-from-drawer (drawer property)
  "Extract PROPERTY value from property DRAWER."
  (let ((nodes (org-element-contents drawer)))
    (cl-loop for node in nodes
             when (and (eq (org-element-type node) 'node-property)
                       (string= (org-element-property :key node) property))
             return (org-element-property :value node))))

(defun prd--parse-buffer-items ()
  "Parse current buffer and return list of `prd-item' structs."
  ;; Need to parse at element level to get property drawers
  (let ((ast (org-element-parse-buffer))
        (items '()))
    (org-element-map ast 'headline
      (lambda (hl)
        (when-let ((todo (org-element-property :todo-keyword hl)))
          (when (member todo prd-todo-keywords)
            (let* ((title (org-element-property :raw-value hl))
                   (begin (org-element-property :begin hl))
                   (line (line-number-at-pos begin))
                   (heading-level (org-element-property :level hl))
                   (custom-id (prd--extract-property hl "CUSTOM_ID"))
                   (agent (prd--extract-property hl "AGENT"))
                   (effort (prd--extract-property hl "EFFORT"))
                   (priority-str (prd--extract-property hl "PRIORITY"))
                   (depends (prd--extract-property hl "DEPENDS"))
                   (blocks (prd--extract-property hl "BLOCKS"))
                   (closed (org-element-property :closed hl)))
              (push (prd-make-item
                     :id custom-id
                     :file (buffer-file-name)
                     :line line
                     :title title
                     :status todo
                     :agent agent
                     :effort effort
                     :priority priority-str
                     :depends (prd--parse-depends depends)
                     :blocks (prd--parse-depends blocks)
                     :properties (prd--extract-all-properties hl)
                     :closed-time (when closed
                                    (org-timestamp-to-time closed))
                     :level heading-level)
                    items))))))
    (nreverse items)))

(defun prd--parse-buffer-categories ()
  "Parse current buffer and return list of `prd-category' structs."
  (let ((ast (org-element-parse-buffer))
        (categories '())
        (cat-regexp (prd--category-id-regexp)))
    (org-element-map ast 'headline
      (lambda (hl)
        (let ((title (org-element-property :raw-value hl)))
          (when (let ((case-fold-search nil))
                  (string-match cat-regexp title))
            (let* ((begin (org-element-property :begin hl))
                   (line (line-number-at-pos begin))
                   (custom-id (prd--extract-property hl "CUSTOM_ID"))
                   (goal (prd--extract-property hl "GOAL"))
                   (depends-cat (prd--extract-property hl "DEPENDS_CATEGORY")))
              (push (prd-make-category
                     :id (or custom-id (match-string 0 title))
                     :file (buffer-file-name)
                     :line line
                     :title title
                     :goal goal
                     :depends-category (prd--parse-depends depends-cat)
                     :items '())
                    categories))))))
    (nreverse categories)))

(defun prd--parse-buffer-checkpoints ()
  "Parse current buffer and return list of `prd-checkpoint' structs.
Finds headings whose title matches `prd-checkpoint-heading-regexp' and
extracts their properties.  Also identifies the parent category for each
checkpoint and collects child items."
  (let ((ast (org-element-parse-buffer))
        (checkpoints '())
        (chk-re prd-checkpoint-heading-regexp))
    (org-element-map ast 'headline
      (lambda (hl)
        (let ((title (org-element-property :raw-value hl)))
          (when (let ((case-fold-search nil))
                  (string-match chk-re title))
            (let* ((begin (org-element-property :begin hl))
                   (level (org-element-property :level hl))
                   (line (line-number-at-pos begin))
                   (custom-id (prd--extract-property hl "CUSTOM_ID"))
                   (criteria (prd--extract-property hl "CRITERIA"))
                   (verify (prd--extract-property hl "VERIFY"))
                   (review-by (prd--extract-property hl "REVIEW_BY"))
                   ;; Derive parent category from CHK ID: CHK-NNN-MM -> NNN
                   (parent-cat (when (and custom-id
                                          (string-match "^CHK-\\([0-9]+\\)" custom-id))
                                 (match-string 1 custom-id))))
              (push (prd-make-checkpoint
                     :id custom-id
                     :file (buffer-file-name)
                     :line line
                     :title title
                     :parent-category parent-cat
                     :criteria criteria
                     :verify verify
                     :review-by review-by
                     :items '()
                     :level level)
                    checkpoints))))))
    (nreverse checkpoints)))

(defun prd--find-parent-checkpoint (file line)
  "Find the checkpoint (CHK-*) that contains LINE in FILE.
Returns the CUSTOM_ID of the checkpoint, or nil if the item is not
nested under a checkpoint."
  (let ((chk-re (concat "^\\*\\* \\(?:CHK-[0-9]+-[0-9]+\\|CHECKPOINT\\) "))
        (cat-re (concat "^\\* \\(?:"
                        (mapconcat #'regexp-quote prd-category-prefixes "\\|")
                        "\\)-[0-9]+")))
    (with-temp-buffer
      (insert-file-contents file)
      (goto-char (point-min))
      (let ((found-chk nil))
        ;; Walk through lines, tracking the most recent CHK heading before LINE
        (while (and (not (eobp)) (<= (line-number-at-pos) line))
          (cond
           ;; A new category heading resets any active checkpoint
           ((looking-at cat-re)
            (setq found-chk nil))
           ;; A checkpoint heading captures its CUSTOM_ID
           ((looking-at chk-re)
            ;; Extract CUSTOM_ID from property drawer below
            (save-excursion
              (when (re-search-forward ":CUSTOM_ID:\\s-+\\(CHK-[^ \t\n]+\\)" nil t)
                (when (<= (line-number-at-pos) line)
                  (setq found-chk (match-string 1))))))
           ;; A level-2 heading that isn't a checkpoint breaks the checkpoint scope
           ((looking-at "^\\*\\* [^*]")
            (unless (looking-at chk-re)
              (setq found-chk nil))))
          (forward-line 1))
        found-chk))))

(defun prd--item-checkpoint-from-boundaries (item boundaries)
  "Find which checkpoint ITEM belongs to using BOUNDARIES.
ITEM is a `prd-item' struct.  BOUNDARIES is a list of (LINE . CHK-ID) pairs.
An item belongs to the most recent checkpoint whose line is before the item's line,
but only if the item is at a deeper heading level (level 3+), since checkpoints
are at level 2.  Items at level 2 are siblings of checkpoints, not children.
Returns the CHK-ID or nil."
  (let ((item-line (prd-item-line item))
        (item-level (or (prd-item-level item) 2))
        (result nil))
    ;; Only items at level 3+ can be children of level-2 checkpoints
    (when (> item-level 2)
      (dolist (boundary boundaries)
        (when (> item-line (car boundary))
          (setq result (cdr boundary)))))
    result))

(defun prd--parse-depends (depends-str)
  "Parse DEPENDS-STR into list of dependency IDs."
  (when (and depends-str (not (string-empty-p depends-str)))
    (mapcar #'string-trim
            (split-string depends-str "[,]" t "[ \t]+"))))

(defun prd--extract-all-properties (headline)
  "Extract all properties from HEADLINE as alist.
Extracts every property from the property drawer, not just required ones.
Uses `org-element-map' to find node-properties regardless of AST nesting."
  (let ((props '()))
    (org-element-map headline 'node-property
      (lambda (node)
        (let ((key (org-element-property :key node))
              (val (org-element-property :value node)))
          (when (and key val (not (string-empty-p val)))
            (push (cons key val) props)))))
    (nreverse props)))

;;; Agent Validation

(defvar prd--agent-cache nil
  "Cache of valid agent references.")

(defun prd--clear-agent-cache ()
  "Clear the agent cache."
  (setq prd--agent-cache nil))

(defun prd--build-agent-cache ()
  "Build cache of valid agent references from agent files."
  (let ((agents (make-hash-table :test 'equal)))
    (dolist (file (prd--agent-files))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((ast (org-element-parse-buffer 'headline))
              (agent-name (file-name-sans-extension
                           (file-name-nondirectory file))))
          ;; Add the agent file itself
          (puthash agent-name file agents)
          ;; Add each section with CUSTOM_ID
          (org-element-map ast 'headline
            (lambda (hl)
              (when-let ((id (prd--extract-custom-id hl)))
                (puthash (format "%s:%s" agent-name id) file agents)))))))
    (setq prd--agent-cache agents)))

(defun prd--valid-agent-p (agent-ref)
  "Check if AGENT-REF is a valid agent reference."
  (unless prd--agent-cache
    (prd--build-agent-cache))
  (when agent-ref
    (let ((ref (prd--extract-agent-from-link agent-ref)))
      (or (gethash ref prd--agent-cache)
          ;; Check if it's just an agent name without section
          (gethash (car (split-string ref ":")) prd--agent-cache)))))

(defun prd--extract-agent-from-link (link-str)
  "Extract agent:section from org link LINK-STR."
  (cond
   ;; Full link: [[file:agents/foo.org::#bar][foo:bar]]
   ((string-match "\\[\\[file:[^]]+\\]\\[\\([^]]+\\)\\]\\]" link-str)
    (match-string 1 link-str))
   ;; Partial link or plain text
   ((string-match "\\([a-z-]+\\)\\(?::\\([a-z-]+\\)\\)?" link-str)
    (if (match-string 2 link-str)
        (format "%s:%s" (match-string 1 link-str) (match-string 2 link-str))
      (match-string 1 link-str)))
   (t link-str)))

(defun prd-list-agents ()
  "List all available agents.
Returns alist of (AGENT-NAME . FILE-PATH)."
  (interactive)
  (unless prd--agent-cache
    (prd--build-agent-cache))
  (let ((agents '()))
    (maphash (lambda (k v)
               (unless (string-match ":" k) ; Only top-level agents
                 (push (cons k v) agents)))
             prd--agent-cache)
    (if (called-interactively-p 'any)
        (message "Available agents: %s"
                 (mapconcat #'car agents ", "))
      agents)))

;;; Dependency Validation

(defvar prd--item-index nil
  "Hash table mapping ITEM IDs to `prd-item' structs.")

(defun prd--setup-org-keywords ()
  "Set up org-mode to recognize PRD TODO keywords.
This is needed because org-element parses TODO keywords from buffer settings,
but our files may not have #+TODO: headers.
Uses buffer-local variable to avoid expensive org-mode reinit."
  (save-excursion
    (goto-char (point-min))
    (unless (re-search-forward "^#\\+TODO:" nil t)
      (goto-char (point-min))
      ;; Skip any existing headers
      (while (looking-at "^#\\+")
        (forward-line 1))
      (insert "#+TODO: ITEM(i) DOING(d) REVIEW(r) | DONE(D) BLOCKED(b)\n")))
  ;; Set buffer-local org-todo-keywords and refresh without full org-mode reinit
  (setq-local org-todo-keywords
              '((sequence "ITEM(i)" "DOING(d)" "REVIEW(r)" "|" "DONE(D)" "BLOCKED(b)")))
  (org-set-regexps-and-options))

(defun prd--build-item-index ()
  "Build index of all items across task files."
  (let ((index (make-hash-table :test 'equal)))
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))  ; org-mode needs this for some features
          (org-mode)
          (prd--setup-org-keywords)
          (dolist (item (prd--parse-buffer-items))
            (when-let ((id (prd-item-id item)))
              (puthash id item index))))))
    (setq prd--item-index index)))

(defun prd--cross-ref-regexp ()
  "Build regexp for cross-category references like PROJ-001:ITEM-005."
  (concat "^\\("
          (mapconcat #'regexp-quote prd-category-prefixes "\\|")
          "\\)-[0-9]+:\\(ITEM-[0-9]+\\)"))

(defun prd--valid-depends-p (depends-list)
  "Check if all dependencies in DEPENDS-LIST are valid."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((cross-ref-re (prd--cross-ref-regexp)))
    (seq-every-p (lambda (dep)
                   (or (gethash dep prd--item-index)
                       ;; Cross-category reference PROJ-XXX:ITEM-YYY
                       (when (string-match cross-ref-re dep)
                         (gethash (match-string 2 dep) prd--item-index))))
                 depends-list)))

(defun prd--detect-cycles ()
  "Detect circular dependencies in items.
Returns list of cycles, each cycle is a list of ITEM IDs."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((visited (make-hash-table :test 'equal))
        (in-stack (make-hash-table :test 'equal))
        (cycles '()))
    (cl-labels
        ((dfs (id path)
           (cond
            ;; Found a cycle - node is already in current recursion stack
            ((gethash id in-stack)
             (let ((cycle-nodes (list id)))
               ;; Extract cycle from path
               (cl-loop for p in path
                        do (push p cycle-nodes)
                        until (equal p id))
               (push (nreverse cycle-nodes) cycles)))
            ;; Already visited in different path, not a cycle on this path
            ((gethash id visited) nil)
            ;; New node - explore
            (t
             (puthash id t visited)
             (puthash id t in-stack)
             (when-let ((item (gethash id prd--item-index)))
               (dolist (dep (prd-item-depends item))
                 (dfs dep (cons id path))))
             (remhash id in-stack)))))
      (maphash (lambda (id _item)
                 (unless (gethash id visited)
                   (dfs id '())))
               prd--item-index))
    cycles))

;;; Validation Functions

(defun prd--validate-item (item)
  "Validate a single ITEM, returning list of errors."
  (let ((errors '())
        (file (prd-item-file item))
        (line (prd-item-line item))
        (cross-ref-re (prd--cross-ref-regexp)))

    ;; Check required properties
    (dolist (prop prd-required-item-properties)
      (unless (cdr (assoc prop (prd-item-properties item)))
        (let* ((prop-lower (downcase prop))
               (hint (cond
                      ((string= prop "CUSTOM_ID")
                       "Add :CUSTOM_ID: ITEM-XXX property")
                      ((string= prop "AGENT")
                       (format "Add :AGENT: property. Available: %s"
                               (mapconcat #'car (prd-list-agents) ", ")))
                      ((string= prop "EFFORT")
                       "Add :EFFORT: property (e.g., 1h, 2h, 30m)")
                      ((string= prop "PRIORITY")
                       "Add :PRIORITY: property (#A, #B, or #C)")
                      (t (format "Add :%s: property" prop)))))
          (push (prd-make-error
                 :file file
                 :line line
                 :rule "required-properties"
                 :severity 'error
                 :message (format "Missing required property: %s" prop)
                 :hint hint
                 :context (prd-item-title item))
                errors))))

    ;; Check CUSTOM_ID format
    (when-let ((id (prd-item-id item)))
      (unless (string-match prd-item-id-regexp id)
        (push (prd-make-error
               :file file
               :line line
               :rule "custom-id-format"
               :severity 'warning
               :message (format "Invalid CUSTOM_ID format: %s" id)
               :hint "Use format ITEM-XXX (e.g., ITEM-001)"
               :context (prd-item-title item))
              errors)))

    ;; Validate AGENT reference
    (when-let ((agent (prd-item-agent item)))
      (unless (prd--valid-agent-p agent)
        (push (prd-make-error
               :file file
               :line line
               :rule "valid-agent-ref"
               :severity 'error
               :message (format "Invalid agent reference: %s" agent)
               :hint (format "Valid agents: %s"
                             (mapconcat #'car (prd-list-agents) ", "))
               :context (prd-item-title item))
              errors)))

    ;; Validate EFFORT format
    (when-let ((effort (prd-item-effort item)))
      (unless (string-match prd-effort-regexp effort)
        (push (prd-make-error
               :file file
               :line line
               :rule "effort-format"
               :severity 'warning
               :message (format "Invalid effort format: %s" effort)
               :hint "Use format Xh or Xm (e.g., 1h, 30m)"
               :context (prd-item-title item))
              errors)))

    ;; Validate DEPENDS references
    (when-let ((deps (prd-item-depends item)))
      (dolist (dep deps)
        (unless (or (gethash dep prd--item-index)
                    (string-match cross-ref-re dep))
          (push (prd-make-error
                 :file file
                 :line line
                 :rule "valid-depends"
                 :severity 'error
                 :message (format "Invalid dependency reference: %s" dep)
                 :hint "Check that the referenced ITEM exists"
                 :context (prd-item-title item))
                errors))))

    ;; I1: Check for TEST_PLAN (warning)
    (when-let ((id (prd-item-id item)))
      (unless (cdr (assoc "TEST_PLAN" (prd-item-properties item)))
        (push (prd-make-error
               :file file
               :line line
               :rule "has-test-plan"
               :severity 'warning
               :message "Missing recommended property: TEST_PLAN"
               :hint "Add :TEST_PLAN: property (e.g., compile, test-rust, e2e)"
               :context (prd-item-title item))
              errors)))

    ;; I1: Check for COMPONENT_REF (info)
    (when-let ((id (prd-item-id item)))
      (unless (cdr (assoc "COMPONENT_REF" (prd-item-properties item)))
        (push (prd-make-error
               :file file
               :line line
               :rule "has-component-ref"
               :severity 'info
               :message "Missing optional property: COMPONENT_REF"
               :hint "Add :COMPONENT_REF: linking to the component/module this implements"
               :context (prd-item-title item))
              errors)))

    (nreverse errors)))

(defun prd--validate-checkpoint (checkpoint)
  "Validate a single CHECKPOINT, returning list of errors."
  (let ((errors '())
        (file (prd-checkpoint-file checkpoint))
        (line (prd-checkpoint-line checkpoint)))

    ;; Check required properties
    (dolist (prop prd-required-checkpoint-properties)
      (let ((has-value
             (pcase prop
               ("CUSTOM_ID" (prd-checkpoint-id checkpoint))
               ("CRITERIA" (prd-checkpoint-criteria checkpoint))
               ("VERIFY" (prd-checkpoint-verify checkpoint))
               (_ nil))))
        (unless (and has-value (not (string-empty-p has-value)))
          (push (prd-make-error
                 :file file
                 :line line
                 :rule "checkpoint-required-properties"
                 :severity 'warning
                 :message (format "Checkpoint missing required property: %s" prop)
                 :hint (format "Add :%s: property to checkpoint heading" prop)
                 :context (prd-checkpoint-title checkpoint))
                errors))))

    ;; Check CUSTOM_ID format
    (when-let ((id (prd-checkpoint-id checkpoint)))
      (unless (let ((case-fold-search nil))
                (string-match prd-checkpoint-id-regexp id))
        (push (prd-make-error
               :file file
               :line line
               :rule "checkpoint-id-format"
               :severity 'warning
               :message (format "Invalid checkpoint CUSTOM_ID format: %s" id)
               :hint "Use format CHK-NNN-MM or CHK-NNN-MM-slug (e.g., CHK-007-01 or CHK-007-01-backend-ready)"
               :context (prd-checkpoint-title checkpoint))
              errors)))

    ;; Check heading level (should be at level 2 within a category)
    (when (and (prd-checkpoint-level checkpoint)
               (/= (prd-checkpoint-level checkpoint) 2))
      (push (prd-make-error
             :file file
             :line line
             :rule "checkpoint-heading-level"
             :severity 'warning
             :message (format "Checkpoint heading should be at level 2 (** ), found level %d"
                              (prd-checkpoint-level checkpoint))
             :hint "Use ** for checkpoint headings within a category (* )"
             :context (prd-checkpoint-title checkpoint))
            errors))

    (nreverse errors)))

(defun prd--validate-file-impl (file)
  "Validate FILE and return list of errors."
  ;; Ensure item index is built for dependency validation
  (unless prd--item-index
    (prd--build-item-index))
  (with-temp-buffer
    (insert-file-contents file)
    (let ((buffer-file-name file))
      (org-mode)
      (prd--setup-org-keywords)
      (let ((items (prd--parse-buffer-items))
            (checkpoints (prd--parse-buffer-checkpoints))
            (errors '()))
        (dolist (item items)
          (setq errors (append errors (prd--validate-item item))))
        (dolist (chk checkpoints)
          (setq errors (append errors (prd--validate-checkpoint chk))))
        errors))))

;;;###autoload
(defun prd-validate-file (file &optional format)
  "Validate FILE and display results.
FORMAT can be `plain' (default) or `json'."
  (interactive
   (list (read-file-name "Validate file: "
                         (prd--tasks-directory))
         'plain))
  (unless prd--item-index
    (prd--build-item-index))
  (let ((errors (prd--validate-file-impl file))
        (format (or format 'plain)))
    (prd--display-validation-results errors format file)))

;;;###autoload
(defun prd-validate-all (&optional format)
  "Validate all @tasks files and display results.
FORMAT can be `plain' (default) or `json'."
  (interactive)
  (prd--clear-agent-cache)
  (prd--build-item-index)
  (let ((all-errors '())
        (format (or format 'plain)))
    (dolist (file (prd--task-org-files))
      (setq all-errors (append all-errors (prd--validate-file-impl file))))
    ;; Check for cycles
    (let ((cycles (prd--detect-cycles)))
      (dolist (cycle cycles)
        (push (prd-make-error
               :file "global"
               :line 0
               :rule "no-circular-deps"
               :severity 'error
               :message (format "Circular dependency detected: %s"
                                (mapconcat #'identity cycle " -> "))
               :hint "Remove one of the dependency links"
               :context nil)
              all-errors)))
    (prd--display-validation-results all-errors format nil)))

;;;###autoload
(defun prd-validate-file-cli (file &optional format)
  "Validate FILE with CLI-friendly output.
FORMAT defaults to `json'."
  (let ((format (or format 'json)))
    (prd-validate-file file format)))

;;;###autoload
(defun prd-validate-all-cli (&optional format)
  "Validate all files with CLI-friendly output.
FORMAT defaults to `json'."
  (let ((format (or format 'json)))
    (prd-validate-all format)))

(defun prd--display-validation-results (errors format &optional file)
  "Display ERRORS in FORMAT.
FILE is optional single file context."
  (let ((result (prd--format-validation-results errors format file)))
    (if (eq format 'json)
        (princ result)
      (if errors
          (with-current-buffer (get-buffer-create "*PRD Validation*")
            (erase-buffer)
            (insert result)
            (goto-char (point-min))
            (special-mode)
            (display-buffer (current-buffer)))
        (message "Validation passed! No errors found.")))))

(defun prd--format-validation-results (errors format &optional file)
  "Format ERRORS according to FORMAT.
FILE is optional context."
  (pcase format
    ('json
     (let ((err-list (seq-filter (lambda (e)
                                   (eq (prd-validation-error-severity e) 'error))
                                 errors))
           (warn-list (seq-filter (lambda (e)
                                    (eq (prd-validation-error-severity e) 'warning))
                                  errors))
           (info-list (seq-filter (lambda (e)
                                    (eq (prd-validation-error-severity e) 'info))
                                  errors))
           (metrics (prd--calculate-metrics)))
       (json-encode
        `((valid . ,(if err-list :json-false t))
          (errors . ,(mapcar #'prd--error-to-alist err-list))
          (warnings . ,(mapcar #'prd--error-to-alist warn-list))
          (info . ,(mapcar #'prd--error-to-alist info-list))
          (needs_link_sync . :json-false)
          (metrics . ((total_items . ,(prd-metrics-total-items metrics))
                      (complete . ,(prd-metrics-complete metrics))
                      (in_progress . ,(prd-metrics-in-progress metrics))
                      (blocked . ,(prd-metrics-blocked metrics))
                      (pending . ,(prd-metrics-pending metrics))))))))
    ('plain
     (if errors
         (with-temp-buffer
           (insert (format "=== PRD Validation Results ===\n\n"))
           (when file
             (insert (format "File: %s\n\n" file)))
           (insert (format "Found %d issue(s):\n\n" (length errors)))
           (dolist (err errors)
             (insert (format "%s in %s:%d\n"
                             (upcase (symbol-name (prd-validation-error-severity err)))
                             (file-name-nondirectory (prd-validation-error-file err))
                             (prd-validation-error-line err)))
             (insert (format "  %s\n" (prd-validation-error-message err)))
             (when (prd-validation-error-hint err)
               (insert (format "  Fix: %s\n" (prd-validation-error-hint err))))
             (when (prd-validation-error-context err)
               (insert (format "  Context: %s\n" (prd-validation-error-context err))))
             (insert "\n"))
           (buffer-string))
       "Validation passed! No errors found.\n"))
    (_ (error "Unknown format: %s" format))))

(defun prd--error-to-alist (err)
  "Convert ERR to alist for JSON serialization."
  `((file . ,(prd-validation-error-file err))
    (line . ,(prd-validation-error-line err))
    (rule . ,(prd-validation-error-rule err))
    (severity . ,(symbol-name (prd-validation-error-severity err)))
    (message . ,(prd-validation-error-message err))
    (hint . ,(or (prd-validation-error-hint err) ""))
    (context . ,(or (prd-validation-error-context err) ""))))

;;; Metrics and Dashboard

(defun prd--calculate-metrics ()
  "Calculate metrics across all items."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((total 0)
        (complete 0)
        (in-progress 0)
        (blocked 0)
        (pending 0))
    (maphash (lambda (_id item)
               (cl-incf total)
               (pcase (prd-item-status item)
                 ("DONE" (cl-incf complete))
                 ("DOING" (cl-incf in-progress))
                 ("REVIEW" (cl-incf in-progress))
                 ("BLOCKED" (cl-incf blocked))
                 (_ (cl-incf pending))))
             prd--item-index)
    (prd-make-metrics
     :total-items total
     :complete complete
     :in-progress in-progress
     :blocked blocked
     :pending pending)))

(defun prd--calculate-category-metrics ()
  "Calculate metrics per category."
  (let ((cat-metrics (make-hash-table :test 'equal)))
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))
          (org-mode)
          (prd--setup-org-keywords)
          (let ((categories (prd--parse-buffer-categories))
                (items (prd--parse-buffer-items)))
            (dolist (cat categories)
              (let ((cat-id (prd-category-id cat)))
                (puthash cat-id
                         `((id . ,cat-id)
                           (title . ,(prd-category-title cat))
                           (file . ,(prd-category-file cat))
                           (items . ,(make-hash-table :test 'equal)))
                         cat-metrics)))
            ;; Associate items with their category (parent)
            (dolist (item items)
              (let* ((item-file (prd-item-file item))
                     (cat-id (prd--find-parent-category item-file (prd-item-line item))))
                (when cat-id
                  (when-let ((cat-data (gethash cat-id cat-metrics)))
                    (puthash (prd-item-id item) item
                             (cdr (assoc 'items cat-data)))))))))))
    cat-metrics))

(defun prd--calculate-category-progress ()
  "Calculate progress for each category.
Returns list of alists with id, title, total, complete, progress,
and optionally checkpoints (list of checkpoint progress alists)."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((progress '())
        (cat-items (make-hash-table :test 'equal))
        ;; Map: chk-id -> (title . items-list)
        (chk-items (make-hash-table :test 'equal))
        ;; Map: cat-id -> list of chk-ids
        (cat-chks (make-hash-table :test 'equal)))
    ;; Group items by their parent category and checkpoint
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))
          (org-mode)
          (prd--setup-org-keywords)
          (let ((categories (prd--parse-buffer-categories))
                (checkpoints (prd--parse-buffer-checkpoints))
                (items (prd--parse-buffer-items)))
            ;; Initialize category entries
            (dolist (cat categories)
              (unless (gethash (prd-category-id cat) cat-items)
                (puthash (prd-category-id cat)
                         `((title . ,(prd-category-title cat))
                           (items . ()))
                         cat-items)))
            ;; Initialize checkpoint entries and link to categories
            (dolist (chk checkpoints)
              (when-let ((chk-id (prd-checkpoint-id chk)))
                (puthash chk-id
                         `((title . ,(prd-checkpoint-title chk))
                           (items . ()))
                         chk-items)
                ;; Link checkpoint to parent category
                (let ((cat-id (prd--find-parent-category file (prd-checkpoint-line chk))))
                  (when cat-id
                    (puthash cat-id
                             (append (gethash cat-id cat-chks '()) (list chk-id))
                             cat-chks)))))
            ;; Associate items with their category and checkpoint
            ;; Build checkpoint line boundaries from parsed checkpoints
            ;; to avoid re-reading the file for each item
            (let ((chk-boundaries
                   (mapcar (lambda (chk)
                             (cons (prd-checkpoint-line chk) (prd-checkpoint-id chk)))
                           checkpoints)))
              (dolist (item items)
                (let ((cat-id (prd--find-parent-category file (prd-item-line item)))
                      (chk-id (prd--item-checkpoint-from-boundaries
                               item chk-boundaries)))
                  (when (and cat-id (gethash cat-id cat-items))
                    (let ((cat-data (gethash cat-id cat-items)))
                      (push item (cdr (assoc 'items cat-data)))))
                  (when (and chk-id (gethash chk-id chk-items))
                    (let ((chk-data (gethash chk-id chk-items)))
                      (push item (cdr (assoc 'items chk-data))))))))))))
    ;; Calculate progress for each category
    (maphash
     (lambda (cat-id cat-data)
       (let* ((items (cdr (assoc 'items cat-data)))
              (title (cdr (assoc 'title cat-data)))
              (total (length items))
              (done (seq-count (lambda (i) (string= (prd-item-status i) "DONE")) items))
              ;; Build checkpoint progress list
              (chk-ids (gethash cat-id cat-chks))
              (chk-progress
               (when chk-ids
                 (mapcar
                  (lambda (chk-id)
                    (let* ((chk-data (gethash chk-id chk-items))
                           (chk-title (cdr (assoc 'title chk-data)))
                           (chk-item-list (cdr (assoc 'items chk-data)))
                           (chk-total (length chk-item-list))
                           (chk-done (seq-count (lambda (i) (string= (prd-item-status i) "DONE"))
                                                chk-item-list)))
                      `((id . ,chk-id)
                        (title . ,chk-title)
                        (total . ,chk-total)
                        (complete . ,chk-done)
                        (progress . ,(if (> chk-total 0) (/ (float chk-done) chk-total) 0.0)))))
                  chk-ids))))
         (push `((id . ,cat-id)
                 (title . ,title)
                 (total . ,total)
                 (complete . ,done)
                 (progress . ,(if (> total 0) (/ (float done) total) 0.0))
                 ,@(when chk-progress
                     `((checkpoints . ,(vconcat chk-progress)))))
               progress)))
     cat-items)
    (nreverse progress)))

(defun prd--find-parent-category (file line)
  "Find the category (PROJ-XXX, BUG-XXX, IMP-XXX) that contains LINE in FILE."
  (let ((cat-re (concat "^\\*+ \\(\\(?:"
                        (mapconcat #'regexp-quote prd-category-prefixes "\\|")
                        "\\)-[0-9]+\\)")))
    (with-temp-buffer
      (insert-file-contents file)
      (goto-char (point-min))
      (let ((found-cat nil))
        (while (re-search-forward cat-re nil t)
          (when (<= (line-number-at-pos) line)
            (setq found-cat (match-string 1))))
        found-cat))))

(defun prd--calculate-agent-metrics ()
  "Calculate metrics per agent."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((agent-metrics (make-hash-table :test 'equal)))
    (maphash
     (lambda (_id item)
       (when-let ((agent-ref (prd-item-agent item)))
         (let* ((agent (prd--extract-agent-from-link agent-ref))
                (agent-name (car (split-string agent ":")))
                (metrics (or (gethash agent-name agent-metrics)
                             (puthash agent-name
                                      (list (cons 'assigned 0) (cons 'complete 0))
                                      agent-metrics))))
           (cl-incf (cdr (assoc 'assigned metrics)))
           (when (string= (prd-item-status item) "DONE")
             (cl-incf (cdr (assoc 'complete metrics)))))))
     prd--item-index)
    agent-metrics))

;;;###autoload
(defun prd-dashboard (&optional format)
  "Generate and display dashboard.
FORMAT can be `plain' (default) or `json'."
  (interactive)
  (prd--build-item-index)
  (let ((format (or format 'plain)))
    (prd--display-dashboard format)))

;;;###autoload
(defun prd-dashboard-cli (&optional format)
  "Generate dashboard with CLI-friendly output.
FORMAT defaults to `json'."
  (let ((format (or format 'json)))
    (prd-dashboard format)))

(defun prd--display-dashboard (format)
  "Display dashboard in FORMAT."
  (let ((result (prd--format-dashboard format)))
    (if (eq format 'json)
        (princ result)
      (with-current-buffer (get-buffer-create "*PRD Dashboard*")
        (erase-buffer)
        (insert result)
        (goto-char (point-min))
        (special-mode)
        (display-buffer (current-buffer))))))

(defun prd--format-dashboard (format)
  "Format dashboard according to FORMAT."
  (let ((metrics (prd--calculate-metrics))
        (agent-metrics (prd--calculate-agent-metrics))
        (cat-progress (prd--calculate-category-progress))
        (blocked-items (prd-list-blocked))
        (velocity-7d (prd--calculate-velocity 7))
        (velocity-trend (prd--velocity-trend 14)))
    (pcase format
      ('json
       (let ((agents-alist '()))
         (maphash (lambda (k v) (push (cons k v) agents-alist))
                  agent-metrics)
         (json-encode
          `((timestamp . ,(format-time-string "%Y-%m-%dT%H:%M:%SZ"))
            (metrics . ((total_items . ,(prd-metrics-total-items metrics))
                        (complete . ,(prd-metrics-complete metrics))
                        (in_progress . ,(prd-metrics-in-progress metrics))
                        (blocked . ,(prd-metrics-blocked metrics))
                        (pending . ,(prd-metrics-pending metrics))))
            (categories . ,(vconcat cat-progress))
            (agents . ,agents-alist)
            (blockers . ,(mapcar (lambda (item)
                                   `((item_id . ,(prd-item-id item))
                                     (blocked_by . ,(prd-item-depends item))))
                                 blocked-items))
            (velocity . ((last_7_days . ,velocity-7d)
                         (trend . ,velocity-trend)))))))
      ('plain
       (with-temp-buffer
         (insert "=== PRD Dashboard ===\n\n")
         (insert (format "Generated: %s\n\n"
                         (format-time-string "%Y-%m-%d %H:%M:%S")))
         (insert "== Overall Metrics ==\n")
         (insert (format "Total Items: %d\n" (prd-metrics-total-items metrics)))
         (insert (format "Complete: %d (%.0f%%)\n"
                         (prd-metrics-complete metrics)
                         (if (> (prd-metrics-total-items metrics) 0)
                             (* 100.0 (/ (float (prd-metrics-complete metrics))
                                         (prd-metrics-total-items metrics)))
                           0)))
         (insert (format "In Progress: %d\n" (prd-metrics-in-progress metrics)))
         (insert (format "Blocked: %d\n" (prd-metrics-blocked metrics)))
         (insert (format "Pending: %d\n\n" (prd-metrics-pending metrics)))

         (insert "== Velocity ==\n")
         (insert (format "Last 7 days: %.1f items/day (%s)\n\n"
                         velocity-7d velocity-trend))

         (when cat-progress
           (insert "== Category Progress ==\n")
           (dolist (cat cat-progress)
             (let ((id (cdr (assoc 'id cat)))
                   (title (cdr (assoc 'title cat)))
                   (total (cdr (assoc 'total cat)))
                   (complete (cdr (assoc 'complete cat)))
                   (progress (cdr (assoc 'progress cat))))
               (insert (format "%s: %d/%d (%.0f%%) - %s\n"
                               id complete total (* 100 progress)
                               (or title "")))))
           (insert "\n"))

         (insert "== Agent Metrics ==\n")
         (maphash (lambda (agent metrics)
                    (insert (format "%s: %d assigned, %d complete\n"
                                    agent
                                    (cdr (assoc 'assigned metrics))
                                    (cdr (assoc 'complete metrics)))))
                  agent-metrics)

         (when blocked-items
           (insert "\n== Blocked Items ==\n")
           (dolist (item blocked-items)
             (insert (format "- %s blocked by: %s\n"
                             (prd-item-id item)
                             (mapconcat #'identity (prd-item-depends item) ", ")))))
         (buffer-string)))
      (_ (error "Unknown format: %s" format)))))

;;;###autoload
(defun prd-quick-status ()
  "Display one-line status summary."
  (interactive)
  (prd--build-item-index)
  (let ((m (prd--calculate-metrics)))
    (message "%d tasks: %d done, %d in-progress, %d blocked, %d pending"
             (prd-metrics-total-items m)
             (prd-metrics-complete m)
             (prd-metrics-in-progress m)
             (prd-metrics-blocked m)
             (prd-metrics-pending m))))

;;; Blocked Tasks

;;;###autoload
(defun prd-list-blocked ()
  "List all blocked tasks."
  (interactive)
  (unless prd--item-index
    (prd--build-item-index))
  (let ((blocked '()))
    (maphash (lambda (_id item)
               (when (string= (prd-item-status item) "BLOCKED")
                 (push item blocked)))
             prd--item-index)
    (if (called-interactively-p 'any)
        (if blocked
            (message "Blocked tasks: %s"
                     (mapconcat (lambda (i) (prd-item-id i)) blocked ", "))
          (message "No blocked tasks."))
      blocked)))

;;; Link Management

(defun prd-item-property (item prop)
  "Get PROP from ITEM's properties alist."
  (cdr (assoc prop (prd-item-properties item))))

;;;###autoload
(defun prd-sync-backlinks ()
  "Synchronize bidirectional links between tasks and documentation."
  (interactive)
  (prd--build-item-index)
  (message "Syncing backlinks...")
  (let ((synced 0)
        (skipped 0))
    (maphash
     (lambda (_id item)
       (dolist (prop '("COMPONENT_REF" "DOC_REF"))
         (when-let ((link (prd-item-property item prop)))
           (if (prd--add-backlink link (prd-item-id item) (prd-item-file item))
               (cl-incf synced)
             (cl-incf skipped)))))
     prd--item-index)
    (message "Synced %d backlinks (%d already existed or failed)." synced skipped)))

(defun prd--add-backlink (target-link item-id item-file)
  "Add backlink to TARGET-LINK pointing to ITEM-ID in ITEM-FILE.
Returns t if backlink was added, nil if it already exists or failed."
  ;; Extract file path and optional target ID from org link
  (when (string-match "\\[\\[file:\\([^]:]+\\)\\(?:::\\(.*\\)\\)?\\]" target-link)
    (let* ((rel-path (match-string 1 target-link))
           (target-id (when (match-string 2 target-link)
                        (replace-regexp-in-string "^#" "" (match-string 2 target-link))))
           (base-dir (file-name-directory
                      (directory-file-name (prd--tasks-directory))))
           (target-file (expand-file-name rel-path base-dir)))
      (when (file-exists-p target-file)
        (with-current-buffer (find-file-noselect target-file)
          (save-excursion
            (goto-char (point-min))
            ;; Find the target headline
            (let ((found (if target-id
                             (org-find-property "CUSTOM_ID" target-id)
                           (point-min))))
              (when found
                (goto-char found)
                ;; Check if backlink already exists
                (let ((existing (org-entry-get nil "IMPLEMENTED_BY")))
                  (if (and existing (string-match (regexp-quote item-id) existing))
                      nil  ; Already exists
                    ;; Add the backlink
                    (let* ((tasks-rel (file-relative-name item-file base-dir))
                           (backlink (format "[[file:%s::#%s][%s]]" tasks-rel item-id item-id))
                           (new-value (if existing
                                          (concat existing ", " backlink)
                                        backlink)))
                      (org-set-property "IMPLEMENTED_BY" new-value)
                      (save-buffer)
                      t)))))))))))

;;;###autoload
(defun prd-audit-links (&optional format)
  "Audit all links and find broken ones.
FORMAT can be `plain' (default) or `json'."
  (interactive)
  (let ((broken '())
        (format (or format 'plain)))
    (dolist (file (prd--all-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (goto-char (point-min))
        (while (re-search-forward "\\[\\[file:\\([^]]+\\)\\]" nil t)
          (let* ((link (match-string 1))
                 (link-path (car (split-string link "::")))
                 (full-path (expand-file-name link-path
                                              (file-name-directory file))))
            (unless (file-exists-p full-path)
              (push `((file . ,file)
                      (line . ,(line-number-at-pos))
                      (link . ,link)
                      (target . ,full-path))
                    broken))))))
    (if (eq format 'json)
        (princ (json-encode `((broken_links . ,broken))))
      (if broken
          (with-current-buffer (get-buffer-create "*PRD Broken Links*")
            (erase-buffer)
            (insert "=== Broken Links ===\n\n")
            (dolist (b broken)
              (insert (format "%s:%d\n  Link: %s\n  Target: %s\n\n"
                              (cdr (assoc 'file b))
                              (cdr (assoc 'line b))
                              (cdr (assoc 'link b))
                              (cdr (assoc 'target b)))))
            (display-buffer (current-buffer)))
        (message "No broken links found.")))))

;;;###autoload
(defun prd-repair-links ()
  "Interactively repair broken links."
  (interactive)
  (prd-audit-links 'plain)
  (when-let ((buf (get-buffer "*PRD Broken Links*")))
    (when (> (buffer-size buf) 0)
      (message "Review broken links in *PRD Broken Links* buffer and fix manually."))))

;;; Effort Parsing

(defun prd--parse-effort-to-minutes (effort-str)
  "Convert EFFORT-STR like '2h' or '30m' to minutes.
Returns nil if the format is invalid."
  (when (and effort-str (string-match "^\\([0-9]+\\)\\([hm]\\)$" effort-str))
    (let ((n (string-to-number (match-string 1 effort-str)))
          (unit (match-string 2 effort-str)))
      (if (string= unit "h") (* n 60) n))))

;;; Velocity Calculation

(defun prd--items-completed-since (days-ago)
  "Return items completed in the last DAYS-AGO days."
  (unless prd--item-index
    (prd--build-item-index))
  (let* ((cutoff (time-subtract (current-time)
                                (days-to-time days-ago)))
         (completed '()))
    (maphash
     (lambda (_id item)
       (when-let ((closed (prd-item-closed-time item)))
         (when (time-less-p cutoff closed)
           (push item completed))))
     prd--item-index)
    completed))

(defun prd--items-completed-between (days-ago-start days-ago-end)
  "Return items completed between DAYS-AGO-START and DAYS-AGO-END days ago.
DAYS-AGO-START is the further-back boundary, DAYS-AGO-END is the more recent."
  (unless prd--item-index
    (prd--build-item-index))
  (let* ((cutoff-start (time-subtract (current-time)
                                      (days-to-time days-ago-start)))
         (cutoff-end (time-subtract (current-time)
                                    (days-to-time days-ago-end)))
         (completed '()))
    (maphash
     (lambda (_id item)
       (when-let ((closed (prd-item-closed-time item)))
         (when (and (time-less-p cutoff-start closed)
                    (not (time-less-p cutoff-end closed)))
           (push item completed))))
     prd--item-index)
    completed))

(defun prd--calculate-velocity (days)
  "Calculate velocity (items/day) over DAYS."
  (let ((completed (prd--items-completed-since days)))
    (if (> days 0)
        (/ (float (length completed)) days)
      0.0)))

(defun prd--velocity-trend (days)
  "Calculate velocity trend comparing two halves of DAYS period.
Compares items completed in second half (more recent) vs first half (earlier)."
  (let* ((half (/ days 2))
         (recent-count (length (prd--items-completed-between half 0)))
         (earlier-count (length (prd--items-completed-between days half)))
         (recent-vel (if (> half 0) (/ (float recent-count) half) 0.0))
         (earlier-vel (if (> half 0) (/ (float earlier-count) half) 0.0)))
    (cond
     ((= earlier-vel 0) (if (> recent-vel 0) "increasing" "unknown"))
     ((> recent-vel (* 1.1 earlier-vel)) "increasing")
     ((< recent-vel (* 0.9 earlier-vel)) "decreasing")
     (t "stable"))))

;;;###autoload
(defun prd-velocity-report (&optional days)
  "Calculate and display velocity report for last DAYS days.
DAYS defaults to 7."
  (interactive "P")
  (prd--build-item-index)
  (let* ((days (or days 7))
         (velocity (prd--calculate-velocity days))
         (trend (prd--velocity-trend (* days 2)))
         (completed (prd--items-completed-since days)))
    (message "Velocity: %.1f items/day (%s) - %d items in %d days"
             velocity trend (length completed) days)))

;;; Burndown Calculation

(defun prd--total-remaining-effort ()
  "Calculate total remaining effort in minutes."
  (unless prd--item-index
    (prd--build-item-index))
  (let ((total 0))
    (maphash
     (lambda (_id item)
       (unless (string= (prd-item-status item) "DONE")
         (when-let ((effort (prd-item-effort item)))
           (when-let ((minutes (prd--parse-effort-to-minutes effort)))
             (cl-incf total minutes)))))
     prd--item-index)
    total))

(defun prd--effort-completed-since (days-ago)
  "Calculate effort completed in the last DAYS-AGO days in minutes."
  (let ((total 0)
        (completed (prd--items-completed-since days-ago)))
    (dolist (item completed)
      (when-let* ((effort (prd-item-effort item))
                  (minutes (prd--parse-effort-to-minutes effort)))
        (cl-incf total minutes)))
    total))

;;;###autoload
(defun prd-burndown (&optional days)
  "Display burndown report.
DAYS specifies the lookback period for velocity calculation (default 7)."
  (interactive "P")
  (prd--build-item-index)
  (let* ((days (or days 7))
         (remaining (prd--total-remaining-effort))
         (completed (prd--effort-completed-since days))
         (velocity-per-day (if (> days 0) (/ (float completed) days) 0))
         (days-to-complete (if (> velocity-per-day 0)
                               (/ remaining velocity-per-day)
                             -1)))
    (with-current-buffer (get-buffer-create "*PRD Burndown*")
      (erase-buffer)
      (insert "=== PRD Burndown Report ===\n\n")
      (insert (format "Generated: %s\n\n"
                      (format-time-string "%Y-%m-%d %H:%M:%S")))
      (insert (format "Remaining effort: %dh %dm\n"
                      (/ remaining 60) (mod remaining 60)))
      (insert (format "Completed last %d days: %dh %dm\n"
                      days (/ completed 60) (mod completed 60)))
      (insert (format "Burn rate: %.1fh/day\n" (/ velocity-per-day 60.0)))
      (if (> days-to-complete 0)
          (insert (format "Projected completion: %.0f days\n" days-to-complete))
        (insert "Projected completion: Unknown (no recent velocity)\n"))
      (special-mode)
      (display-buffer (current-buffer)))))

;;; Cache Management

;;;###autoload
(defun prd-reload-all ()
  "Reload all caches."
  (interactive)
  (prd--clear-agent-cache)
  (setq prd--item-index nil)
  (prd--build-agent-cache)
  (prd--build-item-index)
  (message "Reloaded all caches."))

;;;###autoload
(defun prd-clear-cache ()
  "Clear all caches."
  (interactive)
  (prd--clear-agent-cache)
  (setq prd--item-index nil)
  (message "Cleared all caches."))

;;; Next ID Generation

(defun prd--extract-id-number (id-string prefix)
  "Extract the numeric part from ID-STRING with given PREFIX.
For example, (prd--extract-id-number \"ITEM-045-search-commands\" \"ITEM\") => 45.
Returns nil if ID-STRING is nil or does not match PREFIX."
  (when (and id-string
             (string-match (concat "^" (regexp-quote prefix) "-\\([0-9]+\\)") id-string))
    (string-to-number (match-string 1 id-string))))

(defun prd-next-item-number ()
  "Return the next available ITEM number (max existing + 1).
Always returns at least 1.  Never reuses old numbers (no gap-filling)."
  (prd--build-item-index)
  (let ((max-num 0))
    (maphash (lambda (id _item)
               (let ((num (prd--extract-id-number id "ITEM")))
                 (when (and num (> num max-num))
                   (setq max-num num))))
             prd--item-index)
    (1+ max-num)))

(defun prd-next-category-number (prefix)
  "Return the next available number for category PREFIX.
PREFIX must be one of `prd-category-prefixes' (\"PROJ\", \"BUG\", \"IMP\").
Always returns at least 1.  Never reuses old numbers (no gap-filling)."
  (unless (member prefix prd-category-prefixes)
    (user-error "Invalid category prefix %S; must be one of %S"
                prefix prd-category-prefixes))
  (let ((max-num 0))
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))
          (org-mode)
          (prd--setup-org-keywords)
          (dolist (cat (prd--parse-buffer-categories))
            (let ((num (prd--extract-id-number (prd-category-id cat) prefix)))
              (when (and num (> num max-num))
                (setq max-num num)))))))
    (1+ max-num)))

(defun prd-next-checkpoint-number (category-number)
  "Return the next available checkpoint number for CATEGORY-NUMBER.
CATEGORY-NUMBER is the numeric part of the parent category (e.g., 7 for PROJ-007).
Scans all files for CHK-NNN-MM IDs where NNN matches CATEGORY-NUMBER.
Always returns at least 1.  Never reuses old numbers."
  (let ((max-num 0)
        (prefix (format "CHK-%03d" category-number)))
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))
          (org-mode)
          (prd--setup-org-keywords)
          (dolist (chk (prd--parse-buffer-checkpoints))
            (when-let ((id (prd-checkpoint-id chk)))
              (when (string-prefix-p prefix id)
                ;; Extract MM from CHK-NNN-MM or CHK-NNN-MM-slug
                (when (string-match (concat "^" (regexp-quote prefix) "-\\([0-9]+\\)") id)
                  (let ((num (string-to-number (match-string 1 id))))
                    (when (> num max-num)
                      (setq max-num num))))))))))
    (1+ max-num)))

;;;###autoload
(defun prd-next-ids-cli (&optional format)
  "Return the next available IDs for all prefixes.
FORMAT defaults to `json'.  When FORMAT is `plain', returns human-readable text.
Designed for use via emacsclient.  Includes next CHK numbers per category."
  (let* ((format (or format 'json))
         (next-item (prd-next-item-number))
         (next-proj (prd-next-category-number "PROJ"))
         (next-bug  (prd-next-category-number "BUG"))
         (next-imp  (prd-next-category-number "IMP"))
         ;; Calculate next CHK numbers for existing categories
         (chk-nexts '()))
    ;; Scan for existing category numbers to provide next CHK IDs
    (dolist (file (prd--task-org-files))
      (with-temp-buffer
        (insert-file-contents file)
        (let ((buffer-file-name file))
          (org-mode)
          (prd--setup-org-keywords)
          (dolist (cat (prd--parse-buffer-categories))
            (when-let ((cat-id (prd-category-id cat)))
              (dolist (prefix prd-category-prefixes)
                (when-let ((cat-num (prd--extract-id-number cat-id prefix)))
                  (unless (assoc cat-num chk-nexts)
                    (let ((next-chk (prd-next-checkpoint-number cat-num)))
                      (push (cons cat-num
                                  `((category . ,cat-id)
                                    (next_chk . ,next-chk)
                                    (next_chk_formatted . ,(format "CHK-%03d-%02d" cat-num next-chk))))
                            chk-nexts))))))))))
    (pcase format
      ('json
       (json-encode
        `((next_item . ,next-item)
          (next_item_formatted . ,(format "ITEM-%03d" next-item))
          (next_proj . ,next-proj)
          (next_proj_formatted . ,(format "PROJ-%03d" next-proj))
          (next_bug . ,next-bug)
          (next_bug_formatted . ,(format "BUG-%03d" next-bug))
          (next_imp . ,next-imp)
          (next_imp_formatted . ,(format "IMP-%03d" next-imp))
          (next_chk . ,(mapcar (lambda (entry)
                                 (cdr entry))
                               (nreverse chk-nexts))))))
      ('plain
       (let ((chk-lines (mapconcat
                          (lambda (entry)
                            (let ((data (cdr entry)))
                              (format "Next CHK for %s: %s"
                                      (cdr (assoc 'category data))
                                      (cdr (assoc 'next_chk_formatted data)))))
                          (nreverse chk-nexts)
                          "\n")))
         (concat (format "Next ITEM: %d (ITEM-%03d)\nNext PROJ: %d (PROJ-%03d)\nNext BUG:  %d (BUG-%03d)\nNext IMP:  %d (IMP-%03d)\n"
                         next-item next-item next-proj next-proj
                         next-bug next-bug next-imp next-imp)
                 (when (> (length chk-lines) 0)
                   (concat chk-lines "\n"))))))))

;;; Hooks for Doom Emacs Integration

(defun prd-tasks-setup-doom-hooks ()
  "Set up hooks for Doom Emacs integration."
  (add-hook 'after-save-hook #'prd--after-save-hook))

(defun prd--after-save-hook ()
  "Hook run after saving a file."
  (when (and buffer-file-name
             (string-match "@tasks" buffer-file-name)
             (string-match "\\.org$" buffer-file-name))
    (prd-validate-file buffer-file-name 'plain)))

(provide 'prd-tasks)
;;; prd-tasks.el ends here
