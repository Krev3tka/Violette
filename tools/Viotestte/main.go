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
}

var ExpectedOutputs = map[string]string{
	"bits.vio":          "452",
	"fizzbuzz.vio":      "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz",
	"if_else.vio":       "36\n10.648\n361",
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

	x := (^(0b101 | 0b1011) & 0o53) ^ 0x1E4

	fmt.Println(x)

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
				log.Fatalf("[FAIL] Negative test %s failed.\nExpected error substring: %q\nGot output:\n%s",
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
