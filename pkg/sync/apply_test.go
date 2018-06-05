package sync

import (
	"testing"
)

func TestApplyRootSwap(t *testing.T) {
	// Create a change the swaps out the root entry.
	changes := []*Change{
		{
			Old: testDirectoryEntry,
			New: testFileEntry,
		},
	}

	// Ensure that the swap is applied correctly.
	if result, err := Apply(testDirectoryEntry, changes); err != nil {
		t.Fatal("unable to apply changes:", err)
	} else if !result.Equal(testFileEntry) {
		t.Error("mismatch after root replacement")
	}
}

func TestApplyDiff(t *testing.T) {
	// Compute the diff between two different directories.
	changes := diff("", testDirectoryEntry, testAlternateDirectoryEntry)

	// Test that applying the diff to the base results in the target.
	if result, err := Apply(testDirectoryEntry, changes); err != nil {
		t.Fatal("unable to apply changes:", err)
	} else if !result.Equal(testAlternateDirectoryEntry) {
		t.Error("mismatch after diff/apply cycle")
	}
}

func TestApplyMissingParentPath(t *testing.T) {
	// Create a change with an invalid path.
	changes := []*Change{
		{
			Path: "this/does/not/exist",
			New:  testFileEntry,
		},
	}

	// Ensure that application of the change fails.
	if _, err := Apply(testDirectoryEntry, changes); err == nil {
		t.Fatal("change referencing invalid path did not fail to apply")
	}
}