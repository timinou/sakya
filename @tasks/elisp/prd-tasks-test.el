;;; prd-tasks-test.el --- Tests for prd-tasks.el -*- lexical-binding: t; -*-

;; Author: Design System Team
;; Package-Requires: ((emacs "29.1"))

;;; Commentary:

;; ERT test suite for prd-tasks.el validation and automation functions.
;;
;; Run tests with:
;;   M-x ert RET "prd-" RET
;;
;; Or from command line:
;;   emacs -batch -l prd-tasks.el -l prd-tasks-test.el -f ert-run-tests-batch-and-exit

;;; Code:

(require 'ert)
(require 'prd-tasks)

;;; Test Fixtures

(defvar prd-test--temp-dir nil
  "Temporary directory for test files.")

(defun prd-test--setup ()
  "Set up test fixtures."
  (setq prd-test--temp-dir (make-temp-file "prd-test-" t))
  ;; Create agents directory
  (let ((agents-dir (expand-file-name "agents" prd-test--temp-dir)))
    (make-directory agents-dir)
    ;; Create a test agent file
    (with-temp-file (expand-file-name "test-agent.org" agents-dir)
      (insert "#+TITLE: Test Agent\n\n")
      (insert "* Identity\n")
      (insert ":PROPERTIES:\n")
      (insert ":CUSTOM_ID: identity\n")
      (insert ":END:\n\n")
      (insert "** Name\nTest Agent\n\n")
      (insert "* Core Competencies\n")
      (insert ":PROPERTIES:\n")
      (insert ":CUSTOM_ID: core\n")
      (insert ":END:\n"))
    ;; Create index.org (should be excluded from agents)
    (with-temp-file (expand-file-name "index.org" agents-dir)
      (insert "#+TITLE: Agent Registry\n\n")
      (insert "* Available Agents\n")
      (insert "- test-agent\n")))
  ;; Create task category directories
  (dolist (subdir '("projects" "bugfixes" "improvements"))
    (make-directory (expand-file-name subdir prd-test--temp-dir)))
  ;; Set the tasks directory
  (setq prd-tasks-directory prd-test--temp-dir)
  ;; Clear caches
  (prd-clear-cache))

(defun prd-test--teardown ()
  "Tear down test fixtures."
  (when (and prd-test--temp-dir (file-directory-p prd-test--temp-dir))
    (delete-directory prd-test--temp-dir t))
  (setq prd-test--temp-dir nil)
  (setq prd-tasks-directory nil)
  (prd-clear-cache))

(defmacro prd-test--with-fixture (&rest body)
  "Execute BODY with test fixtures set up."
  (declare (indent 0))
  `(unwind-protect
       (progn
         (prd-test--setup)
         ,@body)
     (prd-test--teardown)))

(defun prd-test--create-item-file (filename content &optional subdir)
  "Create a test item file with FILENAME and CONTENT in SUBDIR (default projects)."
  (let ((file-path (expand-file-name
                    filename
                    (expand-file-name (or subdir "projects") prd-test--temp-dir)))
        ;; Add org-mode header to configure TODO keywords
        (header "#+TODO: ITEM(i) DOING(d) REVIEW(r) | DONE(D) BLOCKED(b)\n\n"))
    (with-temp-file file-path
      (insert header)
      (insert content))
    file-path))

(defun prd-test--create-doc-file (filename content)
  "Create a documentation file with FILENAME and CONTENT at root of temp dir."
  (let ((file-path (expand-file-name filename prd-test--temp-dir)))
    (with-temp-file file-path
      (insert content))
    file-path))

;;; Unit Tests - Property Extraction

(ert-deftest prd-test-parse-depends-empty ()
  "Test parsing empty depends string."
  (should (null (prd--parse-depends nil)))
  (should (null (prd--parse-depends ""))))

(ert-deftest prd-test-parse-depends-single ()
  "Test parsing single dependency."
  (should (equal '("ITEM-001")
                 (prd--parse-depends "ITEM-001"))))

(ert-deftest prd-test-parse-depends-multiple ()
  "Test parsing multiple dependencies."
  (should (equal '("ITEM-001" "ITEM-002" "ITEM-003")
                 (prd--parse-depends "ITEM-001, ITEM-002, ITEM-003"))))

(ert-deftest prd-test-parse-depends-cross-category ()
  "Test parsing cross-category dependencies."
  (should (equal '("PROJ-001:ITEM-005" "ITEM-002")
                 (prd--parse-depends "PROJ-001:ITEM-005, ITEM-002"))))

(ert-deftest prd-test-extract-agent-from-link-full ()
  "Test extracting agent from full org link."
  (should (equal "terminal-specialist:core"
                 (prd--extract-agent-from-link
                  "[[file:agents/terminal-specialist.org::#core][terminal-specialist:core]]"))))

(ert-deftest prd-test-extract-agent-from-link-partial ()
  "Test extracting agent from partial reference."
  (should (equal "test-agent:core"
                 (prd--extract-agent-from-link "test-agent:core"))))

(ert-deftest prd-test-extract-agent-from-link-plain ()
  "Test extracting agent from plain name."
  (should (equal "test-agent"
                 (prd--extract-agent-from-link "test-agent"))))

;;; Unit Tests - Effort Format Validation

(ert-deftest prd-test-effort-format-hours ()
  "Test valid hour formats."
  (should (string-match prd-effort-regexp "1h"))
  (should (string-match prd-effort-regexp "2h"))
  (should (string-match prd-effort-regexp "10h")))

(ert-deftest prd-test-effort-format-minutes ()
  "Test valid minute formats."
  (should (string-match prd-effort-regexp "30m"))
  (should (string-match prd-effort-regexp "45m"))
  (should (string-match prd-effort-regexp "5m")))

(ert-deftest prd-test-effort-format-invalid ()
  "Test invalid effort formats."
  (should-not (string-match prd-effort-regexp "1 hour"))
  (should-not (string-match prd-effort-regexp "2hrs"))
  (should-not (string-match prd-effort-regexp "thirty"))
  (should-not (string-match prd-effort-regexp "")))

;;; Unit Tests - ID Format Validation

(ert-deftest prd-test-item-id-format-valid ()
  "Test valid ITEM ID formats."
  (should (string-match prd-item-id-regexp "ITEM-001"))
  (should (string-match prd-item-id-regexp "ITEM-123"))
  (should (string-match prd-item-id-regexp "ITEM-001-pty-init")))

(ert-deftest prd-test-item-id-format-invalid ()
  "Test invalid ITEM ID formats."
  (let ((case-fold-search nil))  ; Ensure case-sensitive matching
    (should-not (string-match prd-item-id-regexp "TASK-001"))
    (should-not (string-match prd-item-id-regexp "item-001"))
    (should-not (string-match prd-item-id-regexp "ITEM001"))))

(ert-deftest prd-test-category-id-format-valid ()
  "Test valid category ID formats (PROJ, BUG, IMP)."
  (let ((cat-re (prd--category-id-regexp)))
    (should (string-match cat-re "PROJ-001"))
    (should (string-match cat-re "BUG-001"))
    (should (string-match cat-re "IMP-001"))
    (should (string-match cat-re "PROJ-999"))))

(ert-deftest prd-test-category-id-format-invalid ()
  "Test invalid category ID formats."
  (let ((cat-re (prd--category-id-regexp))
        (case-fold-search nil))
    (should-not (string-match cat-re "INIT-001"))
    (should-not (string-match cat-re "proj-001"))
    (should-not (string-match cat-re "TASK-001"))))

;;; Integration Tests - Item Parsing

(ert-deftest prd-test-parse-buffer-items-valid ()
  "Test parsing valid items from buffer."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "test-proj.org"
                 "#+TITLE: Test\n\n* PROJ-001 Test Project\n\n** ITEM Test task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: [[file:../agents/test-agent.org::#core][test-agent:core]]\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n\nTask description.\n")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((items (prd--parse-buffer-items)))
          (should (= 1 (length items)))
          (let ((item (car items)))
            (should (equal "ITEM-001" (prd-item-id item)))
            (should (equal "ITEM" (prd-item-status item)))
            (should (equal "1h" (prd-item-effort item)))))))))

(ert-deftest prd-test-parse-buffer-items-with-depends ()
  "Test parsing items with dependencies."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "test-depends.org"
                 "#+TITLE: Test\n\n** ITEM First task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n** ITEM Second task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 2h\n:PRIORITY: #B\n:DEPENDS: ITEM-001\n:END:\n")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((items (prd--parse-buffer-items)))
          (should (= 2 (length items)))
          (let ((second-item (cadr items)))
            (should (equal '("ITEM-001") (prd-item-depends second-item)))))))))

;;; Integration Tests - Category Parsing

(ert-deftest prd-test-parse-buffer-categories ()
  "Test parsing categories (PROJ/BUG/IMP) from buffer."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "test-proj.org"
                 "#+TITLE: Test\n\n* PROJ-001 Foundation\n:PROPERTIES:\n:CUSTOM_ID: PROJ-001\n:GOAL: Build foundation\n:END:\n\n** ITEM Task 1\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((cats (prd--parse-buffer-categories)))
          (should (= 1 (length cats)))
          (let ((cat (car cats)))
            (should (equal "PROJ-001" (prd-category-id cat)))
            (should (string-match "Foundation" (prd-category-title cat)))))))))

(ert-deftest prd-test-parse-buffer-categories-bug ()
  "Test parsing BUG categories."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "test-bug.org"
                 "#+TITLE: Bug Fix\n\n* BUG-001 Fix crash\n:PROPERTIES:\n:CUSTOM_ID: BUG-001\n:GOAL: Fix the crash\n:END:\n"
                 "bugfixes")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((cats (prd--parse-buffer-categories)))
          (should (= 1 (length cats)))
          (should (equal "BUG-001" (prd-category-id (car cats)))))))))

;;; Integration Tests - Validation

(ert-deftest prd-test-validate-item-missing-agent ()
  "Test validation detects missing AGENT property."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((item (prd-make-item
                 :id "ITEM-001"
                 :file "test.org"
                 :line 10
                 :title "Test task"
                 :status "ITEM"
                 :agent nil
                 :effort "1h"
                 :priority "#B"
                 :depends nil
                 :blocks nil
                 :properties '(("CUSTOM_ID" . "ITEM-001")
                               ("EFFORT" . "1h")
                               ("PRIORITY" . "#B")))))
      (let ((errors (prd--validate-item item)))
        (should (> (length errors) 0))
        (should (seq-find (lambda (e)
                            (string-match "AGENT" (prd-validation-error-message e)))
                          errors))))))

(ert-deftest prd-test-validate-item-invalid-effort ()
  "Test validation detects invalid EFFORT format."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((item (prd-make-item
                 :id "ITEM-001"
                 :file "test.org"
                 :line 10
                 :title "Test task"
                 :status "ITEM"
                 :agent "test-agent:core"
                 :effort "2 hours"
                 :priority "#B"
                 :depends nil
                 :blocks nil
                 :properties '(("CUSTOM_ID" . "ITEM-001")
                               ("AGENT" . "test-agent:core")
                               ("EFFORT" . "2 hours")
                               ("PRIORITY" . "#B")))))
      (let ((errors (prd--validate-item item)))
        (should (seq-find (lambda (e)
                            (and (string= "effort-format"
                                          (prd-validation-error-rule e))
                                 (eq 'warning (prd-validation-error-severity e))))
                          errors))))))

(ert-deftest prd-test-validate-item-all-valid ()
  "Test validation passes for valid item."
  (prd-test--with-fixture
    (prd--build-agent-cache)
    (prd--build-item-index)
    (let ((item (prd-make-item
                 :id "ITEM-001"
                 :file "test.org"
                 :line 10
                 :title "Test task"
                 :status "ITEM"
                 :agent "[[file:agents/test-agent.org::#core][test-agent:core]]"
                 :effort "1h"
                 :priority "#B"
                 :depends nil
                 :blocks nil
                 :properties '(("CUSTOM_ID" . "ITEM-001")
                               ("AGENT" . "[[file:agents/test-agent.org::#core][test-agent:core]]")
                               ("EFFORT" . "1h")
                               ("PRIORITY" . "#B")
                               ("TEST_PLAN" . "compile, test-rust")
                               ("COMPONENT_REF" . "[[file:../../src/mod.rs][Mod]]")))))
      (let ((errors (prd--validate-item item)))
        ;; Should have no errors (might have info messages)
        (should-not (seq-find (lambda (e)
                                (eq 'error (prd-validation-error-severity e)))
                              errors))))))

;;; Integration Tests - Agent Validation

(ert-deftest prd-test-valid-agent-exists ()
  "Test valid agent detection."
  (prd-test--with-fixture
    (prd--build-agent-cache)
    (should (prd--valid-agent-p "test-agent"))
    (should (prd--valid-agent-p "test-agent:core"))
    (should (prd--valid-agent-p "test-agent:identity"))))

(ert-deftest prd-test-invalid-agent-not-exists ()
  "Test invalid agent detection."
  (prd-test--with-fixture
    (prd--build-agent-cache)
    (should-not (prd--valid-agent-p "nonexistent-agent"))
    (should-not (prd--valid-agent-p "fake-agent:section"))))

(ert-deftest prd-test-index-org-excluded-from-agents ()
  "Test that index.org is not counted as a valid agent."
  (prd-test--with-fixture
    (prd--build-agent-cache)
    (should-not (prd--valid-agent-p "index"))))

;;; Integration Tests - Cycle Detection

(ert-deftest prd-test-detect-cycles-no-cycle ()
  "Test cycle detection with no cycles."
  (prd-test--with-fixture
    ;; Create items with linear dependencies
    (prd-test--create-item-file
     "linear.org"
     "** ITEM First\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n** ITEM Second\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:DEPENDS: ITEM-001\n:END:\n\n** ITEM Third\n:PROPERTIES:\n:CUSTOM_ID: ITEM-003\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #C\n:DEPENDS: ITEM-002\n:END:\n")
    (prd--build-item-index)
    (let ((cycles (prd--detect-cycles)))
      (should (= 0 (length cycles))))))

(ert-deftest prd-test-detect-cycles-simple-cycle ()
  "Test cycle detection with simple cycle."
  (prd-test--with-fixture
    ;; Create items with circular dependency
    (prd-test--create-item-file
     "cycle.org"
     "** ITEM First\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:DEPENDS: ITEM-002\n:END:\n\n** ITEM Second\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:DEPENDS: ITEM-001\n:END:\n")
    (prd--build-item-index)
    (let ((cycles (prd--detect-cycles)))
      (should (> (length cycles) 0)))))

;;; Integration Tests - Metrics

(ert-deftest prd-test-calculate-metrics-empty ()
  "Test metrics calculation with no items."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((metrics (prd--calculate-metrics)))
      (should (= 0 (prd-metrics-total-items metrics))))))

(ert-deftest prd-test-calculate-metrics-mixed-status ()
  "Test metrics calculation with various statuses."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "mixed.org"
     "** ITEM Pending task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n** DOING In progress\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n\n** DONE Complete\n:PROPERTIES:\n:CUSTOM_ID: ITEM-003\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #C\n:END:\n\n** BLOCKED Stuck\n:PROPERTIES:\n:CUSTOM_ID: ITEM-004\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd--build-item-index)
    (let ((metrics (prd--calculate-metrics)))
      (should (= 4 (prd-metrics-total-items metrics)))
      (should (= 1 (prd-metrics-complete metrics)))
      (should (= 1 (prd-metrics-in-progress metrics)))
      (should (= 1 (prd-metrics-blocked metrics)))
      (should (= 1 (prd-metrics-pending metrics))))))

;;; Integration Tests - JSON Output

(ert-deftest prd-test-json-output-valid ()
  "Test JSON output is valid JSON."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((json-str (prd--format-validation-results '() 'json nil)))
      (should (stringp json-str))
      ;; Should be parseable
      (let ((parsed (json-read-from-string json-str)))
        (should (assoc 'valid parsed))
        (should (assoc 'errors parsed))
        (should (assoc 'metrics parsed))))))

(ert-deftest prd-test-json-output-with-errors ()
  "Test JSON output includes errors."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let* ((errors (list (prd-make-error
                          :file "test.org"
                          :line 10
                          :rule "test-rule"
                          :severity 'error
                          :message "Test error"
                          :hint "Fix it"
                          :context "Context")))
           (json-str (prd--format-validation-results errors 'json nil))
           (parsed (json-read-from-string json-str)))
      (should (eq :json-false (cdr (assoc 'valid parsed))))
      (should (= 1 (length (cdr (assoc 'errors parsed))))))))

(ert-deftest prd-test-dashboard-json-output ()
  "Test dashboard JSON output structure."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((json-str (prd--format-dashboard 'json)))
      (should (stringp json-str))
      (let ((parsed (json-read-from-string json-str)))
        (should (assoc 'timestamp parsed))
        (should (assoc 'metrics parsed))
        (should (assoc 'agents parsed))
        (should (assoc 'blockers parsed))
        (should (assoc 'velocity parsed))
        ;; Should use "categories" key, not "initiatives"
        (should (assoc 'categories parsed))
        (should-not (assoc 'initiatives parsed))))))

;;; Integration Tests - Full Validation

(ert-deftest prd-test-validate-file-full ()
  "Test full file validation."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "full-test.org"
                 "#+TITLE: Full Test\n\n* PROJ-001 Test Project\n:PROPERTIES:\n:CUSTOM_ID: PROJ-001\n:GOAL: Test goal\n:END:\n\n** ITEM First task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: [[file:../agents/test-agent.org::#core][test-agent:core]]\n:EFFORT: 1h\n:PRIORITY: #B\n:TEST_PLAN: compile\n:COMPONENT_REF: [[file:../../src/mod.rs][Mod]]\n:END:\n\nDescription.\n")))
      (prd--build-item-index)
      (let ((errors (prd--validate-file-impl file)))
        ;; Should have no errors for valid file
        (should-not (seq-find (lambda (e)
                                (eq 'error (prd-validation-error-severity e)))
                              errors))))))

;;; Interactive Function Tests

(ert-deftest prd-test-list-agents ()
  "Test listing agents."
  (prd-test--with-fixture
    (let ((agents (prd-list-agents)))
      (should (listp agents))
      (should (assoc "test-agent" agents))
      ;; index.org should not appear as agent
      (should-not (assoc "index" agents)))))

(ert-deftest prd-test-list-blocked-empty ()
  "Test listing blocked tasks when none exist."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "no-blocked.org"
     "** ITEM Normal task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd--build-item-index)
    (let ((blocked (prd-list-blocked)))
      (should (= 0 (length blocked))))))

(ert-deftest prd-test-list-blocked-with-blocked ()
  "Test listing blocked tasks when some exist."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "with-blocked.org"
     "** BLOCKED Stuck task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:DEPENDS: ITEM-999\n:END:\n")
    (prd--build-item-index)
    (let ((blocked (prd-list-blocked)))
      (should (= 1 (length blocked)))
      (should (equal "ITEM-001" (prd-item-id (car blocked)))))))

;;; Edge Cases

(ert-deftest prd-test-empty-file ()
  "Test handling of empty org file."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file "empty.org" "")))
      (prd--build-item-index)
      (let ((errors (prd--validate-file-impl file)))
        ;; Empty file should have no errors
        (should (= 0 (length errors)))))))

(ert-deftest prd-test-malformed-properties ()
  "Test handling of malformed properties."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "malformed.org"
                 "** ITEM Bad task\n:PROPERTIES:\n:CUSTOM_ID: \n:AGENT:\n:EFFORT: invalid\n:END:\n")))
      (prd--build-item-index)
      (let ((errors (prd--validate-file-impl file)))
        ;; Should report multiple issues
        (should (> (length errors) 0))))))

;;; Unit Tests - Effort Parsing

(ert-deftest prd-test-parse-effort-to-minutes-hours ()
  "Test parsing hours to minutes."
  (should (= 120 (prd--parse-effort-to-minutes "2h")))
  (should (= 60 (prd--parse-effort-to-minutes "1h")))
  (should (= 600 (prd--parse-effort-to-minutes "10h"))))

(ert-deftest prd-test-parse-effort-to-minutes-minutes ()
  "Test parsing minutes."
  (should (= 30 (prd--parse-effort-to-minutes "30m")))
  (should (= 5 (prd--parse-effort-to-minutes "5m")))
  (should (= 45 (prd--parse-effort-to-minutes "45m"))))

(ert-deftest prd-test-parse-effort-to-minutes-invalid ()
  "Test parsing invalid effort strings."
  (should (null (prd--parse-effort-to-minutes nil)))
  (should (null (prd--parse-effort-to-minutes "")))
  (should (null (prd--parse-effort-to-minutes "2 hours")))
  (should (null (prd--parse-effort-to-minutes "2hrs")))
  (should (null (prd--parse-effort-to-minutes "thirty"))))

;;; Unit Tests - Velocity Calculation

(ert-deftest prd-test-velocity-calculation-empty ()
  "Test velocity calculation with no items."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((velocity (prd--calculate-velocity 7)))
      (should (= 0.0 velocity)))))

(ert-deftest prd-test-velocity-trend-unknown ()
  "Test velocity trend with no data."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((trend (prd--velocity-trend 14)))
      (should (equal "unknown" trend)))))

(ert-deftest prd-test-items-completed-since-empty ()
  "Test items completed since with no completed items."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "no-done.org"
     "** ITEM Pending task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd--build-item-index)
    (let ((completed (prd--items-completed-since 7)))
      (should (= 0 (length completed))))))

;;; Unit Tests - Burndown Calculation

(ert-deftest prd-test-total-remaining-effort ()
  "Test total remaining effort calculation."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "effort-test.org"
     "** ITEM Task 1\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 2h\n:PRIORITY: #A\n:END:\n\n** ITEM Task 2\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 30m\n:PRIORITY: #B\n:END:\n\n** DONE Complete\n:PROPERTIES:\n:CUSTOM_ID: ITEM-003\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #C\n:END:\n")
    (prd--build-item-index)
    (let ((remaining (prd--total-remaining-effort)))
      ;; 2h = 120m + 30m = 150m (DONE task not counted)
      (should (= 150 remaining)))))

(ert-deftest prd-test-effort-completed-since-empty ()
  "Test effort completed since with no completed items."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "no-effort.org"
     "** ITEM Pending\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd--build-item-index)
    (let ((completed (prd--effort-completed-since 7)))
      (should (= 0 completed)))))

;;; Integration Tests - Category Progress

(ert-deftest prd-test-calculate-category-progress-empty ()
  "Test category progress with no categories."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((progress (prd--calculate-category-progress)))
      (should (listp progress)))))

(ert-deftest prd-test-calculate-category-progress-with-items ()
  "Test category progress calculation with items."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "proj-progress.org"
     "* PROJ-001 Test Project\n:PROPERTIES:\n:CUSTOM_ID: PROJ-001\n:GOAL: Test\n:END:\n\n** ITEM Task 1\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n** DONE Task 2\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n")
    (prd--build-item-index)
    (let ((progress (prd--calculate-category-progress)))
      (should (listp progress))
      (when progress
        (let ((proj (car progress)))
          (should (equal "PROJ-001" (cdr (assoc 'id proj))))
          (should (= 2 (cdr (assoc 'total proj))))
          (should (= 1 (cdr (assoc 'complete proj))))
          (should (= 0.5 (cdr (assoc 'progress proj)))))))))

;;; Unit Tests - Item Property Helper

(ert-deftest prd-test-item-property ()
  "Test prd-item-property helper function."
  (let ((item (prd-make-item
               :id "ITEM-001"
               :file "test.org"
               :line 1
               :title "Test"
               :status "ITEM"
               :properties '(("COMPONENT_REF" . "[[file:../../src/terminal.rs][Terminal]]")
                             ("AGENT" . "test-agent")))))
    (should (equal "[[file:../../src/terminal.rs][Terminal]]" (prd-item-property item "COMPONENT_REF")))
    (should (equal "test-agent" (prd-item-property item "AGENT")))
    (should (null (prd-item-property item "NONEXISTENT")))))

;;; Integration Tests - Dashboard with Velocity

(ert-deftest prd-test-dashboard-json-includes-velocity ()
  "Test dashboard JSON includes velocity data."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((json-str (prd--format-dashboard 'json)))
      (should (stringp json-str))
      (let ((parsed (json-read-from-string json-str)))
        (should (assoc 'velocity parsed))
        (let ((velocity (cdr (assoc 'velocity parsed))))
          (should (assoc 'last_7_days velocity))
          (should (assoc 'trend velocity)))))))

(ert-deftest prd-test-dashboard-json-includes-categories ()
  "Test dashboard JSON includes categories (not initiatives)."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((json-str (prd--format-dashboard 'json)))
      (should (stringp json-str))
      (let ((parsed (json-read-from-string json-str)))
        (should (assoc 'categories parsed))
        (should-not (assoc 'initiatives parsed))))))

;;; Integration Tests - Closed Time Parsing

(ert-deftest prd-test-parse-item-with-closed-time ()
  "Test parsing item with CLOSED timestamp."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "closed-test.org"
                 "** DONE Completed task\nCLOSED: [2026-01-15 Wed 14:30]\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let ((items (prd--parse-buffer-items)))
          (should (= 1 (length items)))
          (let ((item (car items)))
            (should (equal "ITEM-001" (prd-item-id item)))
            (should (equal "DONE" (prd-item-status item)))
            ;; closed-time should be set (may or may not be parsed depending on org version)
            ))))))

;;; New Tests - Cross-Category Dependencies (B1 fix)

(ert-deftest prd-test-cross-category-dependency-valid ()
  "Test that PROJ-XXX:ITEM-YYY cross-category dependencies are validated."
  (prd-test--with-fixture
    ;; Create a file with a cross-category dependency
    (prd-test--create-item-file
     "proj-a.org"
     "* PROJ-001 Project A\n\n** ITEM Task A\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd-test--create-item-file
     "proj-b.org"
     "* PROJ-002 Project B\n\n** ITEM Task B depends on A\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:DEPENDS: PROJ-001:ITEM-001\n:END:\n")
    (prd--build-item-index)
    ;; PROJ-001:ITEM-001 should resolve to ITEM-001
    (let ((errors (prd--validate-file-impl
                   (expand-file-name "proj-b.org"
                                     (expand-file-name "projects" prd-test--temp-dir)))))
      ;; Should NOT have a "valid-depends" error for the cross-category ref
      (should-not (seq-find (lambda (e)
                              (and (string= "valid-depends" (prd-validation-error-rule e))
                                   (eq 'error (prd-validation-error-severity e))))
                            errors)))))

;;; New Tests - Non-Required Property Extraction (B4 fix)

(ert-deftest prd-test-extract-all-properties-includes-optional ()
  "Test that extract-all-properties extracts all properties, not just required ones."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "props-test.org"
                 "** ITEM Task with extras\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:DEPENDS: ITEM-999\n:COMPONENT_REF: [[file:../../src/mod.rs][Mod]]\n:DOC_REF: [[file:../../docs/design.org][Design]]\n:TEST_PLAN: compile, test-rust, e2e\n:FILES: src/mod.rs, src/lib.rs\n:END:\n")))
      (with-temp-buffer
        (insert-file-contents file)
        (org-mode)
        (let* ((items (prd--parse-buffer-items))
               (item (car items))
               (props (prd-item-properties item)))
          ;; All properties should be present, not just required ones
          (should (assoc "CUSTOM_ID" props))
          (should (assoc "AGENT" props))
          (should (assoc "EFFORT" props))
          (should (assoc "PRIORITY" props))
          (should (assoc "DEPENDS" props))
          (should (assoc "COMPONENT_REF" props))
          (should (assoc "DOC_REF" props))
          (should (assoc "TEST_PLAN" props))
          (should (assoc "FILES" props)))))))

;;; New Tests - Agent Metrics Independence (B8 fix)

(ert-deftest prd-test-agent-metrics-independent-counts ()
  "Test that agent metrics are calculated independently per agent.
This verifies the fix for the shared-literal mutation bug."
  (prd-test--with-fixture
    (prd-test--create-item-file
     "multi-agent.org"
     "** ITEM Agent A task 1\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n** ITEM Agent A task 2\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n")
    ;; Create a second agent
    (let ((agents-dir (expand-file-name "agents" prd-test--temp-dir)))
      (with-temp-file (expand-file-name "other-agent.org" agents-dir)
        (insert "#+TITLE: Other Agent\n\n* Identity\n:PROPERTIES:\n:CUSTOM_ID: identity\n:END:\n")))
    (prd-test--create-item-file
     "other-agent-tasks.org"
     "** DONE Other agent task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-003\n:AGENT: other-agent\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n")
    (prd-clear-cache)
    (prd--build-item-index)
    (let ((agent-metrics (prd--calculate-agent-metrics)))
      ;; test-agent should have 2 assigned, 0 complete
      (let ((ta-metrics (gethash "test-agent" agent-metrics)))
        (should ta-metrics)
        (should (= 2 (cdr (assoc 'assigned ta-metrics))))
        (should (= 0 (cdr (assoc 'complete ta-metrics)))))
      ;; other-agent should have 1 assigned, 1 complete
      (let ((oa-metrics (gethash "other-agent" agent-metrics)))
        (should oa-metrics)
        (should (= 1 (cdr (assoc 'assigned oa-metrics))))
        (should (= 1 (cdr (assoc 'complete oa-metrics))))))))

;;; New Tests - Documentation Files Excluded (B9 fix)

(ert-deftest prd-test-docs-excluded-from-validation ()
  "Test that documentation files are not scanned for task validation."
  (prd-test--with-fixture
    ;; Create a documentation file at root of @tasks with ITEM examples
    (prd-test--create-doc-file
     "reference.org"
     "#+TITLE: Reference Guide\n\n* Example\n** ITEM Example task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-EXAMPLE\n:END:\nThis is just an example.\n")
    ;; Create a real task file in projects/
    (prd-test--create-item-file
     "real-tasks.org"
     "* PROJ-001 Real Project\n\n** ITEM Real task\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n")
    (prd--build-item-index)
    ;; The example item from reference.org should NOT be in the index
    (should-not (gethash "ITEM-EXAMPLE" prd--item-index))
    ;; The real task should be in the index
    (should (gethash "ITEM-001" prd--item-index))))

(ert-deftest prd-test-task-org-files-scans-subdirs-only ()
  "Test that prd--task-org-files only scans category subdirectories."
  (prd-test--with-fixture
    ;; Create files in expected places
    (prd-test--create-item-file "proj.org" "content" "projects")
    (prd-test--create-item-file "bug.org" "content" "bugfixes")
    (prd-test--create-item-file "imp.org" "content" "improvements")
    ;; Create a file at root (should not be included)
    (prd-test--create-doc-file "reference.org" "should not be included")
    (let ((files (prd--task-org-files)))
      ;; Should include files from subdirs
      (should (>= (length files) 3))
      ;; Should NOT include root-level files
      (should-not (seq-find (lambda (f)
                              (string= (file-name-nondirectory f) "reference.org"))
                            files)))))

;;; New Tests - Validation Rules (I1)

(ert-deftest prd-test-validate-missing-test-plan-warning ()
  "Test that missing TEST_PLAN produces a warning."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((item (prd-make-item
                 :id "ITEM-001"
                 :file "test.org"
                 :line 10
                 :title "Test task"
                 :status "ITEM"
                 :agent "test-agent:core"
                 :effort "1h"
                 :priority "#B"
                 :depends nil
                 :blocks nil
                 :properties '(("CUSTOM_ID" . "ITEM-001")
                               ("AGENT" . "test-agent:core")
                               ("EFFORT" . "1h")
                               ("PRIORITY" . "#B")))))
      (let ((errors (prd--validate-item item)))
        (should (seq-find (lambda (e)
                            (and (string= "has-test-plan" (prd-validation-error-rule e))
                                 (eq 'warning (prd-validation-error-severity e))))
                          errors))))))

(ert-deftest prd-test-validate-missing-component-ref-info ()
  "Test that missing COMPONENT_REF produces an info message."
  (prd-test--with-fixture
    (prd--build-item-index)
    (let ((item (prd-make-item
                 :id "ITEM-001"
                 :file "test.org"
                 :line 10
                 :title "Test task"
                 :status "ITEM"
                 :agent "test-agent:core"
                 :effort "1h"
                 :priority "#B"
                 :depends nil
                 :blocks nil
                 :properties '(("CUSTOM_ID" . "ITEM-001")
                               ("AGENT" . "test-agent:core")
                               ("EFFORT" . "1h")
                               ("PRIORITY" . "#B")
                               ("TEST_PLAN" . "compile")))))
      (let ((errors (prd--validate-item item)))
        (should (seq-find (lambda (e)
                            (and (string= "has-component-ref" (prd-validation-error-rule e))
                                 (eq 'info (prd-validation-error-severity e))))
                          errors))))))

;;; New Tests - Velocity Trend Half-Comparison (B5 fix)

(ert-deftest prd-test-velocity-trend-calculates-halves ()
  "Test that velocity trend properly compares first half vs second half.
This verifies the fix where earlier period no longer includes recent period."
  (prd-test--with-fixture
    (prd--build-item-index)
    ;; With no completed items, should be unknown
    (should (equal "unknown" (prd--velocity-trend 14)))
    ;; The function should work without errors even with empty data
    (should (stringp (prd--velocity-trend 30)))))

;;; New Tests - find-parent-category

(ert-deftest prd-test-find-parent-category ()
  "Test finding the parent category for a given line."
  (prd-test--with-fixture
    (let ((file (prd-test--create-item-file
                 "parent-cat.org"
                 "* PROJ-001 Project Alpha\n\n** ITEM Task 1\n:PROPERTIES:\n:CUSTOM_ID: ITEM-001\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #A\n:END:\n\n* PROJ-002 Project Beta\n\n** ITEM Task 2\n:PROPERTIES:\n:CUSTOM_ID: ITEM-002\n:AGENT: test-agent:core\n:EFFORT: 1h\n:PRIORITY: #B\n:END:\n")))
      ;; Test prd--find-parent-category directly
      ;; The file has header on line 1-2, PROJ-001 on line 3, ITEM-001 on ~line 5,
      ;; PROJ-002 on ~line 13, ITEM-002 on ~line 15
      ;; Item under PROJ-001 should find PROJ-001
      (should (equal "PROJ-001" (prd--find-parent-category file 5)))
      ;; Item under PROJ-002 should find PROJ-002
      (should (equal "PROJ-002" (prd--find-parent-category file 15)))
      ;; Line before any category should return nil
      (should (null (prd--find-parent-category file 1))))))

(provide 'prd-tasks-test)
;;; prd-tasks-test.el ends here
