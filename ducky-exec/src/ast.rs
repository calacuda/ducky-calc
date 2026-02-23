use crate::coms_proto::{Command, LedState, SpecialKey};
use alloc::{boxed::Box, string::String, vec::Vec};
use anyhow::bail;
use async_recursion::async_recursion;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};
use log::*;
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;

pub type Result = anyhow::Result<()>;

pub enum ParserEvents {
    Done(
        // Result
    ),
    Error(String),
    /// (line_number, total_lines)
    Line((usize, usize)),
}

#[derive(Clone, Parser)]
#[grammar = "grammar.pest"]
pub struct DuckyScript {
    // port: SerialPortBuilder,
    // pub line: Arc<Mutex<usize>>,
    // event: Sender<'static, CriticalSectionRawMutex, KeyboardReport, 4>,
    kb_out: Sender<'static, CriticalSectionRawMutex, Command, 4>,
    event: Sender<'static, CriticalSectionRawMutex, ParserEvents, 4>,
    len: usize,
}

impl DuckyScript {
    pub fn new(
        kb_out: Sender<'static, CriticalSectionRawMutex, Command, 4>,
        event: Sender<'static, CriticalSectionRawMutex, ParserEvents, 4>,
    ) -> Self {
        Self {
            // port: serialport::new(port_adr, 115200),
            // // line: Arc::new(Mutex::new(0)),
            kb_out,
            event,
            len: 0,
        }
    }

    pub fn get_len(source: &str) -> anyhow::Result<usize> {
        let script = Self::parse(Rule::SCRIPT, source);

        // #[cfg(test)]
        // {
        //
        //     // trace!("flattened script => {:#?}", script?.flatten());
        // }

        debug!("Compiling the source: {:?}", source);
        debug!("script => {:#?}", script);

        Ok(script?.len() - 1)
    }

    pub async fn from_source(&mut self, source: &str) -> Result {
        debug!("getting length...");
        self.len = Self::get_len(source)?;
        debug!("Compiling the source: {:?}", source);
        let script = Self::parse(Rule::SCRIPT, source);
        debug!("script => {:?}", script);

        self.exec(script?).await
    }

    #[async_recursion(?Send)]
    async fn exec(&self, script: Pairs<'async_recursion, Rule>) -> Result {
        for line in script {
            if line.as_rule() == Rule::EOI {
                break;
            } else {
                self.event
                    .send(ParserEvents::Line((line.line_col().0, self.len)))
                    .await;
                info!("on line: {} / {}", line.line_col().0, self.len);
            }

            // self.handle_rule(line).await?;

            if let Err(e) = self.handle_rule(line).await {
                self.event.send(ParserEvents::Error(format!("{e}"))).await;
                error!("got error while parsing ducky script: {e}");
                // self.event.send(ParserEvents::Done(Err(e)));
                // return Ok(());
            }
        }

        self.event.send(ParserEvents::Done()).await;
        Ok(())
    }

    #[async_recursion(?Send)]
    async fn handle_rule(&self, rule: Pair<'async_recursion, Rule>) -> Result {
        match rule.as_rule() {
            Rule::EOI => {}
            Rule::WHITESPACE => {}
            Rule::NEWLINE => {}
            Rule::DELAY => self.delay(rule).await?,
            // Rule::LED => led(line)?,
            Rule::LED_R => self.led(LedState::RED).await,
            Rule::LED_G => self.led(LedState::GREEN).await,
            Rule::LED_OFF => self.led(LedState::OFF).await,
            Rule::LOCK_KEYS => {} // Self::exec(rule.into_inner())?,
            // Rule::STRING => trigger_keys(line)?,
            Rule::STRING => self.exec(rule.into_inner()).await?,
            Rule::STRINGLN => {
                self.exec(rule.into_inner()).await?;
                self.default_delay_key_stroke().await;
                self.hit_key(SpecialKey::Enter).await;
            }
            Rule::STRING_BLOCK => {
                for line in rule.into_inner() {
                    self.handle_rule(line).await?;
                    self.default_delay_key_stroke().await;
                }
            }

            Rule::STRINGLN_BLOCK => {
                for line in rule.into_inner() {
                    self.handle_rule(line).await?;
                    self.default_delay_key_stroke().await;
                    self.hit_key(SpecialKey::Enter).await;
                }
                // stringln_block(line)?,
            }
            Rule::text => self.type_text(rule.as_str()).await,
            Rule::REM
            | Rule::single_line_rem
            | Rule::multi_line_rem
            | Rule::rem_block_start
            | Rule::rem_block_end => {}
            Rule::cursor_keys | Rule::sys_mods | Rule::modifier => {}
            Rule::UP => self.hit_key(SpecialKey::Up).await,
            Rule::DOWN => self.hit_key(SpecialKey::Down).await,
            Rule::LEFT => self.hit_key(SpecialKey::Left).await,
            Rule::RIGHT => self.hit_key(SpecialKey::Right).await,
            Rule::PAGEUP => self.hit_key(SpecialKey::PgUp).await,
            Rule::PAGEDOWN => self.hit_key(SpecialKey::PgDown).await,
            Rule::HOME => self.hit_key(SpecialKey::Home).await,
            Rule::END => self.hit_key(SpecialKey::End).await,
            Rule::INSERT => self.hit_key(SpecialKey::Ins).await,
            Rule::DELETE => self.hit_key(SpecialKey::Del).await,
            Rule::BACKSPACE => self.hit_key(SpecialKey::BackSpace).await,
            Rule::TAB => self.hit_key(SpecialKey::Tab).await,
            Rule::SPACE => self.hit_key(SpecialKey::Space).await,
            Rule::ENTER => self.hit_key(SpecialKey::Enter).await,
            Rule::ESCAPE => self.hit_key(SpecialKey::Esc).await,
            Rule::PAUSE_BREAK => self.hit_key(SpecialKey::PauseBreak).await,
            Rule::PRINTSCREEN => self.hit_key(SpecialKey::PrntScrn).await,
            Rule::MENU_APP => self.hit_key(SpecialKey::Menu).await,
            Rule::F1 => self.hit_key(SpecialKey::F1).await,
            Rule::F2 => self.hit_key(SpecialKey::F2).await,
            Rule::F3 => self.hit_key(SpecialKey::F3).await,
            Rule::F4 => self.hit_key(SpecialKey::F4).await,
            Rule::F5 => self.hit_key(SpecialKey::F5).await,
            Rule::F6 => self.hit_key(SpecialKey::F6).await,
            Rule::F7 => self.hit_key(SpecialKey::F7).await,
            Rule::F8 => self.hit_key(SpecialKey::F8).await,
            Rule::F9 => self.hit_key(SpecialKey::F9).await,
            Rule::F10 => self.hit_key(SpecialKey::F10).await,
            Rule::F11 => self.hit_key(SpecialKey::F11).await,
            Rule::F12 => self.hit_key(SpecialKey::F12).await,
            Rule::SHIFT => self.hit_key(SpecialKey::LeftShift).await,
            Rule::ALT => self.hit_key(SpecialKey::LeftAlt).await,
            Rule::CONTROL => self.hit_key(SpecialKey::LeftCtrl).await,
            Rule::COMMAND | Rule::WINDOWS => self.hit_key(SpecialKey::LeftSuper).await,
            Rule::key_mod_compbo => self.key_mod_combo(rule.into_inner()).await?,
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            // Rule:: => hit_key(SpecialKey::),
            Rule::INJECT_MOD => self.exec(rule.into_inner()).await?,
            Rule::CAPS_LOCK => {
                // self.send_command(Command::TriggerKey(SpecialKey::CapsLock))
                self.send_command(Command::PressKey(SpecialKey::CapsLock))
                    .await
            }
            Rule::NUM_LOCK => {
                // self.send_command(Command::TriggerKey(SpecialKey::NumLock))
                self.send_command(Command::PressKey(SpecialKey::NumLock))
                    .await
            }
            Rule::SCROLLLOCK => {
                // self.send_command(Command::TriggerKey(SpecialKey::ScrollLock))
                self.send_command(Command::PressKey(SpecialKey::ScrollLock))
                    .await
            }
            Rule::DELAY_KW => {}
            Rule::DELAY_ARG => {}
            Rule::INJECT_KW => {}
            Rule::lines | Rule::SCRIPT => {}
        }

        Ok(())
    }

    //  async fn handle_rule(&self, rule: Pin<Box<Pair<'_, Rule>>>) -> Result {
    //     match rule.as_rule() {
    //         Rule::EOI => {}
    //         Rule::WHITESPACE => {}
    //         Rule::NEWLINE => {}
    //         Rule::DELAY => self.delay(*Pin::into_inner(rule)).await?,
    //         // Rule::LED => led(line)?,
    //         Rule::LED_R => self.led(LedState::RED).await?,
    //         Rule::LED_G => self.led(LedState::GREEN).await?,
    //         Rule::LED_OFF => self.led(LedState::OFF).await?,
    //         Rule::LOCK_KEYS => {} // Self::exec(rule.into_inner())?,
    //         // Rule::STRING => trigger_keys(line)?,
    //         Rule::STRING => {
    //             self.exec(Box::pin(Pin::into_inner(rule).deref().clone().into_inner()))
    //                 .await?
    //         }
    //         Rule::STRINGLN => {
    //             self.exec(Box::pin(Pin::into_inner(rule).deref().clone().into_inner()))
    //                 .await?;
    //             self.default_delay_key_stroke().await;
    //             self.hit_key(SpecialKey::Enter).await;
    //         }
    //         Rule::STRING_BLOCK => {
    //             for line in Pin::into_inner(rule).deref().clone().into_inner() {
    //                 self.handle_rule(Box::pin(line)).await?;
    //                 self.default_delay_key_stroke();
    //             }
    //         }
    //
    //         Rule::STRINGLN_BLOCK => {
    //             for line in Pin::into_inner(rule).deref().clone().into_inner() {
    //                 self.handle_rule(Box::pin(line)).await?;
    //                 self.default_delay_key_stroke();
    //                 self.hit_key(SpecialKey::Enter);
    //             }
    //             // stringln_block(line)?,
    //         }
    //         Rule::text => self.type_text(rule.as_str()).await,
    //         Rule::REM
    //         | Rule::single_line_rem
    //         | Rule::multi_line_rem
    //         | Rule::rem_block_start
    //         | Rule::rem_block_end => {}
    //         Rule::cursor_keys | Rule::sys_mods | Rule::modifier => {}
    //         Rule::UP => self.hit_key(SpecialKey::Up).await,
    //         Rule::DOWN => self.hit_key(SpecialKey::Down).await,
    //         Rule::LEFT => self.hit_key(SpecialKey::Left).await,
    //         Rule::RIGHT => self.hit_key(SpecialKey::Right).await,
    //         Rule::PAGEUP => self.hit_key(SpecialKey::PgUp).await,
    //         Rule::PAGEDOWN => self.hit_key(SpecialKey::PgDown).await,
    //         Rule::HOME => self.hit_key(SpecialKey::Home).await,
    //         Rule::END => self.hit_key(SpecialKey::End).await,
    //         Rule::INSERT => self.hit_key(SpecialKey::Ins).await,
    //         Rule::DELETE => self.hit_key(SpecialKey::Del).await,
    //         Rule::BACKSPACE => self.hit_key(SpecialKey::BackSpace).await,
    //         Rule::TAB => self.hit_key(SpecialKey::Tab).await,
    //         Rule::SPACE => self.hit_key(SpecialKey::Space).await,
    //         Rule::ENTER => self.hit_key(SpecialKey::Enter).await,
    //         Rule::ESCAPE => self.hit_key(SpecialKey::Esc).await,
    //         Rule::PAUSE_BREAK => self.hit_key(SpecialKey::PauseBreak).await,
    //         Rule::PRINTSCREEN => self.hit_key(SpecialKey::PrntScrn).await,
    //         Rule::MENU_APP => self.hit_key(SpecialKey::Menu).await,
    //         Rule::F1 => self.hit_key(SpecialKey::F1).await,
    //         Rule::F2 => self.hit_key(SpecialKey::F2).await,
    //         Rule::F3 => self.hit_key(SpecialKey::F3).await,
    //         Rule::F4 => self.hit_key(SpecialKey::F4).await,
    //         Rule::F5 => self.hit_key(SpecialKey::F5).await,
    //         Rule::F6 => self.hit_key(SpecialKey::F6).await,
    //         Rule::F7 => self.hit_key(SpecialKey::F7).await,
    //         Rule::F8 => self.hit_key(SpecialKey::F8).await,
    //         Rule::F9 => self.hit_key(SpecialKey::F9).await,
    //         Rule::F10 => self.hit_key(SpecialKey::F10).await,
    //         Rule::F11 => self.hit_key(SpecialKey::F11).await,
    //         Rule::F12 => self.hit_key(SpecialKey::F12).await,
    //         Rule::SHIFT => self.hit_key(SpecialKey::LeftShift).await,
    //         Rule::ALT => self.hit_key(SpecialKey::LeftAlt).await,
    //         Rule::CONTROL => self.hit_key(SpecialKey::LeftCtrl).await,
    //         Rule::COMMAND | Rule::WINDOWS => self.hit_key(SpecialKey::LeftSuper).await,
    //         Rule::key_mod_compbo => {
    //             self.key_mod_combo(Pin::into_inner(rule).deref().clone().into_inner())
    //                 .await?
    //         }
    //         // Rule:: => hit_key(SpecialKey::),
    //         // Rule:: => hit_key(SpecialKey::),
    //         // Rule:: => hit_key(SpecialKey::),
    //         // Rule:: => hit_key(SpecialKey::),
    //         Rule::INJECT_MOD => {
    //             self.exec(Box::pin(Pin::into_inner(rule).deref().clone().into_inner()))
    //                 .await?
    //         }
    //         Rule::CAPS_LOCK => {
    //             self.send_command(Command::TriggerKey(SpecialKey::CapsLock))
    //                 .await
    //         }
    //         Rule::NUM_LOCK => {
    //             self.send_command(Command::TriggerKey(SpecialKey::NumLock))
    //                 .await
    //         }
    //         Rule::SCROLLLOCK => {
    //             self.send_command(Command::TriggerKey(SpecialKey::ScrollLock))
    //                 .await
    //         }
    //         Rule::DELAY_KW => {}
    //         Rule::DELAY_ARG => {}
    //         Rule::INJECT_KW => {}
    //         Rule::lines | Rule::SCRIPT => {}
    //     }
    //
    //     Ok(())
    // }

    async fn key_mod_combo(&self, rules: Pairs<'_, Rule>) -> Result {
        let mut to_release: Vec<SpecialKey> = Vec::with_capacity(rules.len());

        for rule in rules {
            let key = match rule.as_rule() {
                Rule::UP => self.press_key(SpecialKey::Up).await,
                Rule::DOWN => self.press_key(SpecialKey::Down).await,
                Rule::LEFT => self.press_key(SpecialKey::Left).await,
                Rule::RIGHT => self.press_key(SpecialKey::Right).await,
                Rule::PAGEUP => self.press_key(SpecialKey::PgUp).await,
                Rule::PAGEDOWN => self.press_key(SpecialKey::PgDown).await,
                Rule::HOME => self.press_key(SpecialKey::Home).await,
                Rule::END => self.press_key(SpecialKey::End).await,
                Rule::INSERT => self.press_key(SpecialKey::Ins).await,
                Rule::DELETE => self.press_key(SpecialKey::Del).await,
                Rule::BACKSPACE => self.press_key(SpecialKey::BackSpace).await,
                Rule::TAB => self.press_key(SpecialKey::Tab).await,
                Rule::SPACE => self.press_key(SpecialKey::Space).await,
                Rule::ENTER => self.press_key(SpecialKey::Enter).await,
                Rule::ESCAPE => self.press_key(SpecialKey::Esc).await,
                Rule::PAUSE_BREAK => self.press_key(SpecialKey::PauseBreak).await,
                Rule::PRINTSCREEN => self.press_key(SpecialKey::PrntScrn).await,
                Rule::MENU_APP => self.press_key(SpecialKey::Menu).await,
                Rule::F1 => self.press_key(SpecialKey::F1).await,
                Rule::F2 => self.press_key(SpecialKey::F2).await,
                Rule::F3 => self.press_key(SpecialKey::F3).await,
                Rule::F4 => self.press_key(SpecialKey::F4).await,
                Rule::F5 => self.press_key(SpecialKey::F5).await,
                Rule::F6 => self.press_key(SpecialKey::F6).await,
                Rule::F7 => self.press_key(SpecialKey::F7).await,
                Rule::F8 => self.press_key(SpecialKey::F8).await,
                Rule::F9 => self.press_key(SpecialKey::F9).await,
                Rule::F10 => self.press_key(SpecialKey::F10).await,
                Rule::F11 => self.press_key(SpecialKey::F11).await,
                Rule::F12 => self.press_key(SpecialKey::F12).await,
                Rule::SHIFT => self.press_key(SpecialKey::LeftShift).await,
                Rule::ALT => self.press_key(SpecialKey::LeftAlt).await,
                Rule::CONTROL => self.press_key(SpecialKey::LeftCtrl).await,
                Rule::COMMAND | Rule::WINDOWS => self.press_key(SpecialKey::LeftSuper).await,
                Rule::text => {
                    self.type_text(rule.as_str()).await;

                    None
                }
                _ => None,
            };

            if let Some(key) = key {
                to_release.push(key);
            }

            self.default_delay_key_stroke().await
        }

        to_release.reverse();

        for key in to_release.into_iter() {
            self.send_command(Command::ReleaseKey(key)).await;
        }

        Ok(())
    }

    async fn send_command(&self, cmd: Command) {
        // if let Ok(message) = serde_json::to_string(&cmd) {
        //     if let Ok(mut port) = self.port.clone().open_native() {
        //         let mut encoded: Vec<u8> = message.as_bytes().to_vec();
        //         encoded.push('\n' as u8);
        //
        //         if let Err(e) = port.write_all(&encoded) {
        //             error!("sending data over uart failed with error: {e}");
        //         }
        //     } else {
        //         error!("failed to open serial port");
        //     }
        // } else {
        //     error!("failed to serialize command");
        // }

        debug!("was instructed to {cmd:?}");
        self.kb_out.send(cmd).await;
    }

    async fn hit_key(&self, key: SpecialKey) {
        self.send_command(Command::PressKey(key)).await;
        self.default_delay_key_stroke().await;
        self.send_command(Command::ReleaseKey(key)).await;
    }

    async fn press_key(&self, key: SpecialKey) -> Option<SpecialKey> {
        self.send_command(Command::PressKey(key)).await;

        Some(key)
    }

    async fn type_text(&self, text: &str) {
        for char in text.as_bytes().into_iter() {
            self.send_command(Command::PressChar(*char as char)).await;
            self.default_delay_up_down().await;
            self.send_command(Command::ReleaseChar(*char as char)).await;
        }
    }

    async fn led(&self, state: LedState) {
        // println!("setting led to be {state:?}");
        self.send_command(Command::LED(state)).await;

        // Ok(())
    }

    async fn delay(&self, line: Pair<'_, Rule>) -> Result {
        if let Some(delay_amt) = line.into_inner().next() {
            if let Ok(time) = delay_amt.as_str().parse() {
                // sleep(Duration::from_millis(time));
                Timer::after(Duration::from_millis(time)).await;
                Ok(())
            } else {
                bail!("delay must be a positive whole number and nothing else.");
            }
        } else {
            bail!("delay takes a argument");
        }
    }

    async fn default_delay_up_down(&self) {
        // sleep(Duration::from_millis(25));
        Timer::after(Duration::from_millis(1)).await;
    }

    async fn default_delay_key_stroke(&self) {
        // sleep(Duration::from_millis(50));
        Timer::after(Duration::from_millis(1)).await;
    }
}
