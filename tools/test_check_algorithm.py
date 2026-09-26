"""Focused checks must fail visibly instead of reporting unverified success."""

import contextlib
import io
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import check_algorithm


class AlgorithmChecksTests(unittest.TestCase):
    def test_reduction_runs_direct_algorithms_and_source_integration(self):
        with patch.object(check_algorithm, "run") as run:
            with patch.object(check_algorithm, "library_artifact",
                              return_value=Path("target/debug/libcocycle.rlib")):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(check_algorithm.main(["persistence-reduction"]), 0)
        commands = [call.args[0] for call in run.call_args_list]
        tests = [command for command in commands if command[:2] == ["cargo", "test"]]
        self.assertTrue(any("--lib" in command and "persistence::" in command
                            for command in tests))
        self.assertTrue(any(all(name in command for name in
                               ("filtered_complex", "prime_fields", "rips_resources", "rips_api"))
                            for command in tests))
        self.assertTrue(any(command[0] == "rustdoc" and
                            "docs/development/persistence-reduction.md" in command
                            for command in commands))

    def test_distances_run_public_contracts_private_oracles_and_example(self):
        with patch.object(check_algorithm, "run") as run:
            with patch.object(check_algorithm, "library_artifact",
                              return_value=Path("target/debug/libcocycle.rlib")):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(check_algorithm.main(["diagram-distances"]), 0)
        commands = [call.args[0] for call in run.call_args_list]
        tests = [command for command in commands if command[:2] == ["cargo", "test"]]
        self.assertTrue(any("--test" in command and "diagram_distances" in command
                            and "--example" in command for command in tests))
        self.assertTrue(any("--lib" in command and "diagram_distances::" in command
                            for command in tests))
        self.assertFalse(any("filtered_complex" in command for command in tests))

    def test_construction_selects_the_actual_example_tests_and_tutorial(self):
        with patch.object(check_algorithm, "run") as run:
            with patch.object(check_algorithm, "library_artifact",
                              return_value=Path("target/debug/libcocycle.rlib")):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(check_algorithm.main(["complex-construction"]), 0)
        commands = [call.args[0] for call in run.call_args_list]
        tests = next(command for command in commands if command[:2] == ["cargo", "test"])
        self.assertEqual(tests[tests.index("--example") + 1], "complex_construction")
        self.assertIn("filtered_complex", tests)
        self.assertNotIn("descriptors", tests)
        self.assertTrue(any(command[0] == "rustdoc"
                            and "docs/development/complex-construction.md" in command
                            for command in commands))

    def test_failed_step_stops_before_later_checks(self):
        failure = subprocess.CalledProcessError(7, ["check_source"])
        with patch.object(check_algorithm, "run", side_effect=failure) as run:
            with contextlib.redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(check_algorithm.main(["diagram-analysis"]), 1)
        self.assertEqual(run.call_count, 1)
        self.assertIn("Algorithm checks failed", errors.getvalue())

    def test_missing_tool_is_a_reported_failure(self):
        with patch.object(check_algorithm, "run", side_effect=FileNotFoundError("cargo")):
            with contextlib.redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(check_algorithm.main(["diagram-analysis"]), 1)
        self.assertIn("cargo", errors.getvalue())

    def test_artifact_uses_cargos_path_and_requires_library_output(self):
        output = ('{"reason":"compiler-artifact","target":{"name":"cocycle"},'
                  '"filenames":["custom target/debug/libcocycle.rlib"]}\n')
        with patch.object(check_algorithm, "run", return_value=subprocess.CompletedProcess(
                [], 0, stdout=output)):
            self.assertEqual(check_algorithm.library_artifact(),
                             Path("custom target/debug/libcocycle.rlib"))
        with patch.object(check_algorithm, "run", return_value=subprocess.CompletedProcess(
                [], 0, stdout='{"reason":"build-finished","success":true}\n')):
            with self.assertRaisesRegex(ValueError, "library artifact"):
                check_algorithm.library_artifact()


if __name__ == "__main__":
    unittest.main()
