use std::fs;

use crate::styled_text::StyledText;
use indoc::indoc;
use sysinfo::System;

const LOGOS: &[(&str, &str, &str)] = &[
    (
        "windows",
        indoc! {r#"
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////

            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
            /////////////////  /////////////////
        "#},
        "bright_blue",
    ),
    (
        "macos",
        indoc! {r#"
                                 ..'
                             ,xNMM.
                           .OMMMMo
                           lMM"
                 .;loddo:.  .olloddol;.
               cKMMMMMMMMMMNWMMMMMMMMMM0:
             .KMMMMMMMMMMMMMMMMMMMMMMMWd.
             XMMMMMMMMMMMMMMMMMMMMMMMX.
            ;MMMMMMMMMMMMMMMMMMMMMMMM:
            :MMMMMMMMMMMMMMMMMMMMMMMM:
            .MMMMMMMMMMMMMMMMMMMMMMMMX.
             kMMMMMMMMMMMMMMMMMMMMMMMMWd.
             'XMMMMMMMMMMMMMMMMMMMMMMMMMMk
              'XMMMMMMMMMMMMMMMMMMMMMMMMK.
                kMMMMMMMMMMMMMMMMMMMMMMd
                 ;KMMMMMMMWXXWMMMMMMMk.
                   "cooc*"    "*coo'"
        "#},
        "bright_yellow",
    ),
    (
        "darwin",
        indoc! {r#"
                                 ..'
                             ,xNMM.
                           .OMMMMo
                           lMM"
                 .;loddo:.  .olloddol;.
               cKMMMMMMMMMMNWMMMMMMMMMM0:
             .KMMMMMMMMMMMMMMMMMMMMMMMWd.
             XMMMMMMMMMMMMMMMMMMMMMMMX.
            ;MMMMMMMMMMMMMMMMMMMMMMMM:
            :MMMMMMMMMMMMMMMMMMMMMMMM:
            .MMMMMMMMMMMMMMMMMMMMMMMMX.
             kMMMMMMMMMMMMMMMMMMMMMMMMWd.
             'XMMMMMMMMMMMMMMMMMMMMMMMMMMk
              'XMMMMMMMMMMMMMMMMMMMMMMMMK.
                kMMMMMMMMMMMMMMMMMMMMMMd
                 ;KMMMMMMMWXXWMMMMMMMk.
                   "cooc*"    "*coo'"
        "#},
        "bright_yellow",
    ),
    (
        "arch",
        indoc! {r#"
                              -`
                             .o+`
                            `ooo/
                           `+oooo:
                          `+oooooo:
                          -+oooooo+:
                        `/:-:++oooo+:
                       `/++++/+++++++:
                      `/++++++++++++++:
                     `/+++ooooooooooooo/`
                    ./ooosssso++osssssso+`
                   .oossssso-````/ossssss+`
                  -osssssso.      :ssssssso.
                 :osssssss/        osssso+++.
                /ossssssss/        +ssssooo/-
              `/ossssso+/:-        -:/+osssso+-
             `+sso+:-`                 `.-/+oso:
            `++:.                           `-/+/
            .`                                 `/
        "#},
        "cyan",
    ),
    (
        "ubuntu",
        indoc! {r#"
                                         ....
                          .',:clooo:  .:looooo:.
                       .;looooooooc  .oooooooooo'
                    .;looooool:,''.  :ooooooooooc
                   ;looool;.         'oooooooooo,
                  ;clool'             .cooooooc.  ,,
                     ...                ......  .:oo,
              .;clol:,.                        .loooo'
             :ooooooooo,                        'ooool
            'ooooooooooo.                        loooo.
            'ooooooooool                         coooo.
             ,loooooooc.                        .loooo.
               .,;;;'.                          ;ooooc
                   ...                         ,ooool.
                .cooooc.              ..',,'.  .cooo.
                  ;ooooo:.           ;oooooooc.  :l.
                   .coooooc,..      coooooooooo.
                     .:ooooooolc:. .ooooooooooo'
                       .':loooooo;  ,oooooooooc
                           ..';::c'  .;loooo:'
        "#},
        "red",
    ),
    (
        "debian",
        indoc! {r#"
                    _,met$$$$$gg.
                 ,g$$$$$$$$$$$$$$$P.
               ,g$$P""       """Y$$.".
              ,$$P'              `$$$.
            ',$$P       ,ggs.     `$$b:
            `d$$'     ,$P"'   .    $$$
             $$P      d$'     ,    $$P
             $$:      $$.   -    ,d$$'
             $$;      Y$b._   _,d$P'
             Y$$.    `.`"Y$$$$P"'
             `$$b      "-.__
              `Y$$b
               `Y$$.
                 `$$b.
                   `Y$$b.
                     `"Y$b._
                         `""""
        "#},
        "white",
    ),
    (
        "fedora",
        indoc! {r#"
                         .',;::::;,'.
                     .';:cccccccccccc:;,.
                  .;cccccccccccccccccccccc;.
                .:cccccccccccccccccccccccccc:.
              .;ccccccccccccc;⠞      ⠳;ccccccc;.
             .:ccccccccccccc;          ;ccccccc:.
            .:ccccccccccccc;    ;cc;    ;ccccccc:.
            ,cccccccccccccc;   .;cc;;  :;cccccccc,
            :cccccccccccccc;   .;cccccccccccccccc:
            :ccccccc;˙˙˙˙˙˙;       .;cccccccccccc:
            cccccc;        ;       .;cccccccccccc;
            ccccc;    ;cccc;   .;cccccccccccccccc'
            ccccc;   ;ccccc;   .;ccccccccccccccc;
            ccccc;    .ccc.    ;ccccccccccccccc;
            cccccc;          :;cccccccccccccc:,
            cccccccc;.:   :.;cccccccccccccc:,.
            ccccccccccccccccccccccccccccc:'.
            :ccccccccccccccccccccccc:;,..
             ':cccccccccccccccc::;,.
        "#},
        "bright_blue",
    ),
    (
        "nixos",
        indoc! {r#"
                      ▗▄▄▄       ▗▄▄▄▄    ▄▄▄▖
                      ▜███▙       ▜███▙  ▟███▛
                       ▜███▙       ▜███▙▟███▛
                        ▜███▙       ▜██████▛
                 ▟█████████████████▙ ▜████▛     ▟▙
                ▟███████████████████▙ ▜███▙    ▟██▙
                       ▄▄▄▄▖           ▜███▙  ▟███▛
                      ▟███▛             ▜██▛ ▟███▛
                     ▟███▛               ▜▛ ▟███▛
            ▟███████████▛                  ▟██████████▙
            ▜██████████▛                  ▟███████████▛
                  ▟███▛ ▟▙               ▟███▛
                 ▟███▛ ▟██▙             ▟███▛
                ▟███▛  ▜███▙           ▝▀▀▀▀
                ▜██▛    ▜███▙ ▜██████████████████▛
                 ▜▛     ▟████▙ ▜████████████████▛
                       ▟██████▙         ▜███▙
                      ▟███▛▜███▙         ▜███▙
                     ▟███▛  ▜███▙         ▜███▙
                     ▝▀▀▀    ▀▀▀▀▘         ▀▀▀▘
        "#},
        "bright_blue",
    ),
    (
        "linux",
        indoc! {r#"
                     _nnnn_
                    dGGGGMMb
                   @p~qp~~qMb
                   M|@||@) M|
                   @,----.JM|
                  JS^\__/  qKL
                 dZP        qKRb
                dZP          qKKb
               fZP            SMMb
               HZM            MMMM
               FqM            MMMM
             __| ".        |\dS"qML
             |    `.       | `' \Zq
            _)      \.___.,|     .'
            \____   )MMMMMP|   .'
                 `-'       `--' hjm
        "#},
        "white",
    ),
    (
        "fetchall",
        indoc! {r#"
            '########:'########:'########::'######::'##::::'##:
             ##.....:: ##.....::... ##..::'##... ##: ##:::: ##:
             ##::::::: ##:::::::::: ##:::: ##:::..:: ##:::: ##:
             ######::: ######:::::: ##:::: ##::::::: #########:
             ##...:::: ##...::::::: ##:::: ##::::::: ##.... ##:
             ##::::::: ##:::::::::: ##:::: ##::: ##: ##:::: ##:
             ##::::::: ########:::: ##::::. ######:: ##:::: ##:
            ..::::::::........:::::..::::::......:::..:::::..::
              ::::::'###::::'##:::::::'##:::::::::::::
              :::::'## ##::: ##::::::: ##:::::::::::::
              ::::'##:. ##:: ##::::::: ##:::::::::::::
              :::'##:::. ##: ##::::::: ##:::::::::::::
              ::: #########: ##::::::: ##:::::::::::::
              ::: ##.... ##: ##::::::: ##:::::::::::::
              ::: ##:::: ##: ########: ########:::::::
              :::..:::::..::........::........::::::::
        "#},
        "bright_magenta",
    ),
];

pub const OS_LOGOS: &[&str] = &[
    "windows", "macos", "darwin", "arch", "ubuntu", "debian", "fedora", "nixos", "linux",
];

fn lookup_logo(name: &str) -> Option<(&'static str, &'static str)> {
    LOGOS
        .iter()
        .find(|(logo_name, _, _)| logo_name.eq_ignore_ascii_case(name))
        .map(|(_, art, colour)| (*art, *colour))
}

pub fn get_logo(mode: &str, colour: &str) -> StyledText {
    if mode == "auto" {
        let os_name: String = System::name().unwrap_or_else(|| "Unknown".to_string());
        let (logo, logo_colour) = lookup_logo(&os_name).unwrap_or_else(|| {
            if std::env::consts::OS == "linux" {
                lookup_logo("linux").unwrap_or(lookup_logo("fetchall").unwrap_or(("", "white")))
            } else {
                lookup_logo("fetchall").unwrap_or(("", "white"))
            }
        });
        let colour = if colour != "auto" {
            colour
        } else {
            logo_colour
        };
        StyledText::new(logo).apply_style(colour)
    } else if mode == "none" {
        StyledText::new("")
    } else if let Some((logo, logo_colour)) = lookup_logo(mode) {
        let colour = if colour != "auto" {
            colour
        } else {
            logo_colour
        };
        StyledText::new(logo).apply_style(colour)
    } else if let Ok(read) = fs::read_to_string(mode) {
        StyledText::new(&read).apply_style(colour)
    } else {
        let (logo, logo_colour) = lookup_logo("fetchall").unwrap_or(("", "white"));
        let colour = if colour != "auto" {
            colour
        } else {
            logo_colour
        };
        StyledText::new(logo).apply_style(colour)
    }
}
