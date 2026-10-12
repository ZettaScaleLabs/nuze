#!/usr/bin/env nu

const cwd = path self | path dirname

def "x test" [] {
    cd $cwd

    cargo build --message-format=short --manifest-path ($cwd | path join Cargo.toml) -p nuze --bin nuze
    let nuze = $cwd | path join target debug nuze

    const always_exclude = "**/_*.nu"

    for test in (glob tests/**/*.nu --exclude [$always_exclude experimental/**]) {
        info $test
        try {
            ^$nuze -0 $test
        } catch {|err|
            info ko $test $nuze
            continue
        }

        info ok $test
    }

    for test in (glob tests/experimental/**/*.nu --exclude [$always_exclude]) {
        info $test
        try {
            ^$nuze -X0 $test
        } catch {|err|
            info ko $test $nuze
            continue
        }

        info ok $test
    }
}

def info [test] {
    print -e $"(ansi yellow_bold)TEST:(ansi reset) ($test | path relative-to $"($cwd)/tests")"
}

def "info ko" [test nuze -x] {
    if $x {
        print -e $"(ansi red_bold)KO:(ansi reset) ($test | path relative-to $"($cwd)/tests") \(experimental\); rerun with `($nuze) -X0 ($test)`"
    } else {
        print -e $"(ansi red_bold)KO:(ansi reset) ($test | path relative-to $"($cwd)/tests"); rerun with `($nuze) -0 ($test)`"
    }
}

def "info ok" [test -x] {
    if $x {
        print -e $"(ansi green_bold)OK:(ansi reset) ($test | path relative-to $"($cwd)/tests") \(experimental\)"
    } else {
        print -e $"(ansi green_bold)OK:(ansi reset) ($test | path relative-to $"($cwd)/tests")"
    }
}
