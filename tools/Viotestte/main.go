package main

import (
	"fmt"
	"io/fs"
	"log"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

const TestPaths = "../../examples/"

var ExpectedErrors = map[string]string{
	"fail_breaking_bad.vio":      "found `break` outside of a loop",
	"fail_constatation.vio":      "couldn't assign again to const variable",
	"fail_not_full_return.vio":   "there's no `return` in every path",
	"fail_redefine_fun_vars.vio": "couldn't re-define",
	"fail_user_struct.vio":       "couldn't re-define",
	"fail_with_no_extend.vio":    "no method named `distance` found for type `Point`",
}

var ExpectedOutputs = map[string]string{
	"bits.vio":            "452",
	"demo_showcase.vio":   "3.16228\n8\nIs `Violette` empty?: false",
	"escape_analysis.vio": "Quotes: \"Hello, Violette!\"\nBackslash: \\",
	"extern_fun.vio":      "64\n8\n3",
	"extend_demo.vio":     "5",
	"factorial.vio":       "120\n1\n1",
	"fibonacci.vio":       "55",
	"field_assigning.vio": "15",
	"fizzbuzz.vio":        "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz",
	"if_else.vio":         "36\n10.648\n361",
	"moduling.vio":        "17",
	"multiplication_table_via_ranges.vio": "1 2 3 4 5 6 7 8 9 \n" +
		"2 4 6 8 10 12 14 16 18 \n" +
		"3 6 9 12 15 18 21 24 27 \n" +
		"4 8 12 16 20 24 28 32 36 \n" +
		"5 10 15 20 25 30 35 40 45 \n" +
		"6 12 18 24 30 36 42 48 54 \n" +
		"7 14 21 28 35 42 49 56 63 \n" +
		"8 16 24 32 40 48 56 64 72 \n" +
		"9 18 27 36 45 54 63 72 81 ",
	"point.vio":          "3.5",
	"sprouting.vio":      "true",
	"square.vio":         "36",
	"string_concat.vio":  "Hello, Violette!",
	"to_be_continue.vio": "0\n1\n2\n3\n4\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16",
	"string_indexation.vio": "86\nDoes `Violette` start with `Vio`: true\nDoes `Violette` start with `Vim`: false\nDoes `Violette` ends with `lette`: true\nDoes `Violette` ends with `latte`: false",
	"zero_init.vio":      "Bio: \n0",
}

var ansiRegex = regexp.MustCompile(`\x1b\[[0-9;]*[a-zA-Z]`)

func stripAnsi(str string) string {
	return ansiRegex.ReplaceAllString(str, "")
}

var testCases []TestCase

type TestCase struct {
	Path           string
	ExpectedError  string
	ExpectedOutput string
	IsNegativeTest bool
}

func runCompiler(filePath string) (string, error) {
	cmd := exec.Command("../../target/debug/violette", "run", filePath)

	output, err := cmd.CombinedOutput()

	cleanStr := stripAnsi(strings.TrimSpace(string(output)))

	return cleanStr, err
}

func WalkDirFunc(path string, d fs.DirEntry, err error) error {
	if err != nil {
		return err
	}

	if d.IsDir() {
		return nil
	}

	if filepath.Ext(path) == ".vio" {
		filename := filepath.Base(path)
		directory := filepath.Dir(path)

		if strings.Contains(directory, "/benching") || strings.HasPrefix(filename, "bench_") {
		    return nil
		}

		if filename == "input.vio" {
			return nil
		}

		isNegative := strings.Contains(path, "/invalid/") && strings.HasPrefix(filename, "fail_")

		expectedOutput, ok := ExpectedOutputs[filename]
		expectedErr, errOk := ExpectedErrors[filename]

		if !ok && !errOk {
			return fmt.Errorf("failed to find right output or error for %s", filename)
		}

		testCases = append(testCases, TestCase{
			Path:           path,
			ExpectedOutput: expectedOutput,
			ExpectedError:  expectedErr,
			IsNegativeTest: isNegative,
		})
	}

	return nil
}

func main() {
    buildCmd := exec.Command("cargo", "build", "--quiet")
	if err := buildCmd.Run(); err != nil {
		log.Fatalf("Failed to build Violette: %v", err)
	}

	err := filepath.WalkDir(TestPaths, WalkDirFunc)

	if err != nil {
		log.Fatalf("Walking Directories error: %v", err)
	}

	fmt.Printf("Running %d integration tests for Violette...\n", len(testCases))
	passed := 0

	for _, test := range testCases {
		output, err := runCompiler(test.Path)

		if test.IsNegativeTest {
			if !strings.Contains(output, test.ExpectedError) {
				log.Fatalf("[FAIL] Negative test %s failed.\nExpected error substring: %q\nGot output:\n%s",
					test.Path, test.ExpectedError, output)
			}
			fmt.Printf("  [PASS] %s (caught %q)\n", filepath.Base(test.Path), test.ExpectedError)
			passed++
		} else {
			if err != nil {
				log.Fatalf("[FAIL] Positive test %s failed to compile/run:\nErr: %v\nOutput:\n%s",
					test.Path, err, output)
			}

			if strings.TrimSpace(output) != strings.TrimSpace(test.ExpectedOutput) {
				log.Fatalf("[FAIL] Output mismatch for %s.\nExpected:\n%s\nGot:\n%s",
					test.Path, test.ExpectedOutput, output)
			}
			fmt.Printf("  [PASS] %s\n", filepath.Base(test.Path))
			passed++
		}
	}

	fmt.Printf("\nAll %d tests passed successfully\n", passed)
}
