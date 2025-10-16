use parslay::prelude::*;
use parslay::reactive::{RwSignal, SignalGet, SignalUpdate};
use parslay::widgets::input::InputExt;

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        let flight_return = RwSignal::new(false);
        let start = RwSignal::new("2026.03.28".to_string());
        let end = RwSignal::new("2026.03.28".to_string());
        let result = RwSignal::new("".to_string());
        let book_button = button("Book")
            .on_click(move |_, _| {
                if date_check(start.get()) {
                    let return_message = if flight_return.get() && date_check(end.get()) {
                        format!(", return on {end}")
                    } else {
                        "".to_string()
                    };
                    result.set(format!("Flight booked on {start}{return_message}"))
                }
            })
            .background_color(Rgba::GRAY);
        let return_date = dyn_input(move || format!("{end}"))
            .on_edit({
                let book_button = book_button.clone();
                move |this| {
                    end.set(this.get_text());
                    if date_check(this.get_text()) {
                        this.set_background_color(Rgba::WHITE);
                    } else {
                        this.set_background_color(Rgba::hex("#a00000").unwrap());
                    }
                    book_button.set_disabled(
                        !date_check(start.get())
                            || !date_check(this.get_text())
                            || this.get_text().cmp(&start.get()).is_lt(),
                    )
                }
            })
            .padding(4);

        vstack(
            4,
            (
                drop_down("one-way flight", "return flight").on_edit({
                    let return_date = return_date.clone();
                    move |d| {
                        let do_return = d.get_text() == "return flight";
                        flight_return.set(do_return);
                        return_date.set_disabled(!do_return);
                    }
                }),
                dyn_input(move || format!("{start}"))
                    .on_edit({
                        let book_button = book_button.clone();
                        move |this| {
                            start.set(this.get_text());
                            if date_check(this.get_text()) {
                                this.set_background_color(Rgba::WHITE);
                            } else {
                                this.set_background_color(Rgba::hex("#a00000").unwrap());
                            }
                            book_button.set_disabled(!date_check(this.get_text()));
                        }
                    })
                    .padding(4),
                return_date,
                book_button,
                dyn_label(move || format!("{result}")),
            ),
        )
        .frame(FrameType::Frame)
        .background_color(Rgba::WHITE)
        .padding(8)
    })?;
    Ok(())
}

fn date_check(text: String) -> bool {
    const DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut parts = text.split('.');
    let Some(yr_str) = parts.next() else {
        return false;
    };
    let yr = match yr_str.parse::<u32>() {
        Ok(yr) if (2025..=2999).contains(&yr) => yr,
        _ => return false,
    };
    let Some(mo_str) = parts.next() else {
        return false;
    };
    let mo = match mo_str.parse::<u32>() {
        Ok(mo) if mo > 0 && mo <= 12 && (yr > 2025 || mo >= 10) => mo,
        _ => return false,
    };
    let Some(day_str) = parts.next() else {
        return false;
    };
    matches!(day_str.parse::<u32>(), Ok(day) if day > 0
                && day
                    <= DAYS[mo as usize - 1] as u32
                        + (mo == 2) as u32 * ((yr % 4 == 0 && yr % 100 > 0) || yr % 400 == 0) as u32)
        && parts.next().is_none()
}
