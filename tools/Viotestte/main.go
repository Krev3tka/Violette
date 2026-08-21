package main

import (
	"fmt"
	"io/fs"
	"log"
	"os/exec"
	"path/filepath"
	"strings"
)

const TestPaths = "../../examples/"

var Errors = map[string]string{
	"fail_user_struct.vio": "Type errors: [DuplicateDefinition(\"User\")]",
	"fail_constatation.vio": "Type errors: [AssignmentToImmutable { name: \"THREE_HOURS_IN_SECONDS\", kind: Const }, " +
		"AssignmentToImmutable { name: \"x\", kind: Let }]",
}

var ExpectedOutputs = map[string]string{
	"bits.vio":     "452",
	"escape_analysis.vio": "Quotes: \"Hello, Violette!\"" + "\nBackslash: \\",
    "factorial.vio": "120\n1\n1",
    "fibonacci.vio": "55",
	"fizzbuzz.vio": "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz",
	"if_else.vio":  "36\n10.648\n361",
	"multiplication_table_via_ranges.vio": "1 2 3 4 5 6 7 8 9 \n" +
		"2 4 6 8 10 12 14 16 18 \n" +
		"3 6 9 12 15 18 21 24 27 \n" +
		"4 8 12 16 20 24 28 32 36 \n" +
		"5 10 15 20 25 30 35 40 45 \n" +
		"6 12 18 24 30 36 42 48 54 \n" +
		"7 14 21 28 35 42 49 56 63 \n" +
		"8 16 24 32 40 48 56 64 72 \n" +
		"9 18 27 36 45 54 63 72 81 ",
	"point.vio":         "3.5",
	"sprouting.vio":     "true",
	"square.vio":        "36",
	"string_concat.vio": "Hello, Violette!",
}

var testCases []TestCase

type TestCase struct {
	Path           string
	ExpectedError  string
	ExpectedOutput string
	IsNegativeTest bool
}

func runCompiler(filePath string) (string, error) {
	cmd := exec.Command("cargo", "run", "--quiet", "--", "run", filePath)

	output, err := cmd.CombinedOutput()

	output = []byte(strings.Trim(string(output), "\n"))

	return string(output), err
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

		if filename == "input.vio" {
		    return nil
		}

		isNegative := strings.Contains(path, "/invalid/") && strings.HasPrefix(filename, "fail_")

		expectedOutput, ok := ExpectedOutputs[filename]
		expectedErr, errOk := Errors[filename]

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
	err := filepath.WalkDir(TestPaths, WalkDirFunc)

	if err != nil {
		log.Fatalf("Walking Directories error: %v", err)
	}

	fmt.Printf("How many tests: %d\n", len(testCases))
	for _, test := range testCases {
		fmt.Printf("- %s (Negative: %t, Expected: %q)\n", test.Path, test.IsNegativeTest, test.ExpectedError)

		output, err := runCompiler(test.Path)

		if test.IsNegativeTest {
			if !strings.Contains(output, test.ExpectedError) {
				log.Fatalf("[FAIL] Negative test %s failed.\nExpected error substring: %q\nGot output:\n%q",
					test.Path, test.ExpectedError, output)
			}
			fmt.Printf("[PASS] Negative test %s correctly failed with expected message.\n", test.Path)
		} else {
			if err != nil {
				log.Fatalf("[FAIL] Positive test %s failed to compile/run:\nErr: %v\nOutput:\n%s",
					test.Path, err, output)
			}

			if output != test.ExpectedOutput {
				log.Fatalf("[FAIL] Positive test %s failed.\nExpected: %q, got: %q",
					test.Path, test.ExpectedOutput, output)
			} else {
				fmt.Printf("[PASS] Positive test %s succeeded.\n", test.Path)
			}
		}
	}
}
