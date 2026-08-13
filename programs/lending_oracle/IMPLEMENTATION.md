# Реализация: Repay, Withdraw, SPL-интеграция (кратко и по делу)

Ниже — понятный пошаговый план работ, зачем они нужны и какие файлы/функции править.

## Цель
- Сделать базовый рабочий lending-flow: deposit → borrow (выдача) → repay → withdraw.
- Подключить простой SPL-поток для эмуляции выдачи/погашения (USDC).

## Почему это важно
- Сейчас у проекта есть только учёт чисел в `UserAccount` (deposit/credit). Чтобы протокол был полезен, нужно:
  - реально выдавать/принимать активы (SPL) или корректно симулировать их при тестах;
  - возможность погасить долг (`repay`), иначе `borrow` — бессмысленно;
  - возможность вывести залог (`withdraw`) при условии, что позиция остаётся healthy (LTV).

## Новые инструкции (файлы)
- `instructions/repay.rs` — принять сумму от пользователя, уменьшить `UserAccount.credit`. Если долг полностью погашен — разрешить withdraw.
- `instructions/withdraw.rs` — позволить снять часть/весь `UserAccount.deposit` при условии, что после снятия `debt <= collateral * price * ltv`.

Файлы, которые нужно обновить:
- `instructions/borrow.rs` — добавить реальный transfer USDC в ветке успеха (CPI к `spl-token`). Сейчас там только update state.
- `instructions/deposit.rs` — при желании сохранить прием в token-ATA вместо lamports (по архитектуре можно оставить SOL как коллатерал, но учесть decimals).
- `state/user_account.rs` — подумать над переименованием: `deposit -> collateral`, `credit -> debt`. Добавить поля `collateral_mint`/`debt_mint`/`user_collateral_ata`/`user_debt_ata` или просто комментарии к ним.

## Детали реализации (repay)
1. Accounts:
   - `user: Signer`
   - `user_account: Account<UserAccount>` (has_one = user, seeds)
   - `protocol_usdc_escrow: AccountInfo` (PDA holding USDC if you use real tokens)
   - `user_usdc_ata: AccountInfo` (user token account)
   - `token_program: Program<'_, Token>`
2. Flow:
   - CPI transfer `user_usdc_ata` -> `protocol_usdc_escrow` of `amount` (if real tokens)
   - Decrease `user_account.credit = credit.checked_sub(amount)` (use checked math)
   - Emit event `Repay { user, amount, remaining_debt }`

## Детали реализации (withdraw)
1. Accounts:
   - `user: Signer`
   - `user_account: Account<UserAccount>`
   - `maybe_vault_pda` or `user_collateral_account` (откуда переводить SOL или collateral token)
   - `system_program` (для lamports) или `token_program` (SPL)
2. Flow:
   - Рассчитать новую collateral = deposit - withdraw_amount
   - Нормализовать: `collateral_value_usd = collateral * price / SOL_DECIMALS` (u128, checked)
   - Проверка: `debt <= collateral_value_usd * ltv / 100` — если нарушится, отказ
   - Перевести средства пользователю (lamports или token transfer)
   - Уменьшить `user_account.deposit` на `withdraw_amount`

## SPL: простая интеграция для тестовой выдачи/погашения
1. Решение: использовать тестовый mint USDC в локальном net или создать PDA-escrow и authority.
2. Шаги:
   - В root/instructions добавь accounts для mint/escrow/ATA
   - В `borrow` после успешной проверки LTV сделать `spl_token::instruction::transfer` через CPI
   - В `repay` сделать обратный transfer от user ATA в protocol ATA
   - Для безопасности — использовать PDA как authority и `CpiContext::new_with_signer`

Пример вызова CPI (схема):

```rust
let cpi_accounts = spl_token::instruction::Transfer { /* fields */ };
let cpi_program = ctx.accounts.token_program.to_account_info();
let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
token::transfer(cpi_ctx, amount)?;
```

*(Точный код зависит от того, какие `AccountInfo`/`Account` ты пропишешь в `#[derive(Accounts)]`)*

## PDAs и authority
- Если выдаёшь реальные токены — заводишь PDA `protocol_authority` (seed: `b"authority"`) и PDA `protocol_escrow` для хранения USDC. Mint должен доверять этому PDA для операций (либо использовать mint, которым владеешь в тестах).
- Для переводов из escrow нужен `CpiContext::new_with_signer(..., signer_seeds)`.

## Математика и нормализация (важно)
- Работать в `u128` и использовать `checked_*` для всех операций.
- Нормализация примера (SOL 9 decimals, USDC 6 decimals, price with exponent):
  - `collateral_lamports: u128` → `collateral_sol = collateral_lamports / 10^9`
  - `price` (Pyth) вероятно с экспонентом `-8` или др.; привести price к `price_scaled` с нужным множителем
  - `collateral_value_usd = collateral_lamports * price_adjusted / 10^9` (в u128)

## Тесты (минимум)
- Unit/integration ts-tests (Anchor):
  1. init_config
  2. init_user
  3. deposit
  4. borrow (проверить: debt увеличился, USDC был выдан если интегрирован)
  5. repay (частичный, полный)
  6. withdraw (успешный и отказной)
  7. liquidate (если debt > max)

Команды для тестирования локально (пример):

```bash
# поднять локальный validator
anchor localnet --reset
# запустить тесты
anchor test
```

## Файловый TODO (конкретно)
- `programs/lending_oracle/src/instructions/repay.rs` — создать и зарегистрировать в `instructions/mod.rs` и `lib.rs`.
- `programs/lending_oracle/src/instructions/withdraw.rs` — то же.
- `programs/lending_oracle/src/instructions/borrow.rs` — добавить token CPI ветку (комментарии/placeholder пока можно поставить).
- `programs/lending_oracle/src/instructions/deposit.rs` — при желании перенести прием на token-ATA.
- `programs/lending_oracle/src/state/user_account.rs` — рассмотреть переименование полей и добавить ATA-поля (опционально).
- `programs/lending_oracle/README.md` или `IMPLEMENTATION.md` — (этот файл) держать в репозитории.

## Риски и замечания
- Перед интеграцией Pyth убедись, что у тебя есть тестовый фид или fallback; не используйте сырой Pyth price без проверки staleness/confidence.
- Проверки авторизаций и `has_one` обязательны для safety.

---
Если хочешь, завтра могу прислать готовые skeleton-файлы `repay.rs` и `withdraw.rs` (без полного CPI), чтобы ты мог дописать сам — скажи, хочешь ли skeleton или полный пример с SPL CPI.
