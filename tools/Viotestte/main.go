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
var testCases []TestCase

type TestCase struct {
	Path           string
	ExpectedError  string
	IsNegativeTest bool
}

func runCompiler(filePath string) (string, error) {
	cmd := exec.Command("cargo", "run", "--", "run", filePath)

	output, err := cmd.CombinedOutput()
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

		testCases = append(testCases, TestCase{
			Path:           path,
			ExpectedError:  Errors[filename],
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
				log.Fatalf("[FAIL] Negative test %s failed.\nExpected error substring: %q\nGot output:\n%s",
					test.Path, test.ExpectedError, output)
			}
			fmt.Printf("[PASS] Negative test %s correctly failed with expected message.\n", test.Path)
		} else {
			if err != nil {
				log.Fatalf("[FAIL] Positive test %s failed to compile/run:\nErr: %v\nOutput:\n%s",
					test.Path, err, output)
			}
			fmt.Printf("[PASS] Positive test %s succeeded.\n", test.Path)
		}
	}
}
