for test in (cat ./failing-tests.txt)
    cargo test $test
end
