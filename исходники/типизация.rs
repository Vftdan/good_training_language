use super::Результат;
use диагностика::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ПримТип {
    Цел1,
    Цел8,
    Строка,
    Лог,
}

impl ПримТип {
    pub fn текст(&self) -> &str {
        match self {
            ПримТип::Цел1 => "цел(1)",
            ПримТип::Цел8 => "цел(8)",
            ПримТип::Строка => "строка",
            ПримТип::Лог => "лог",
        }
    }

    pub fn размер(&self) -> usize {
        match self {
            ПримТип::Цел1 => 1,
            ПримТип::Цел8 => 8,
            ПримТип::Строка => 16, // Два 64-х битных числа: указатель на начало и размер.
            ПримТип::Лог => 8,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Тип {
    ПримТип(ПримТип),
    Массив { прим_тип: ПримТип, размер: usize },
    Срез { прим_тип: ПримТип },
}

impl Тип {
    pub fn текст(&self) -> String {
        match self {
            Тип::ПримТип(тип) => тип.текст().to_string(),
            Тип::Массив {прим_тип, размер} => format!("массив({размер}) {прим_тип}", прим_тип = прим_тип.текст()),
            Тип::Срез {прим_тип} => format!("срез {прим_тип}", прим_тип = прим_тип.текст()),
        }
    }

    pub fn размер(&self) -> usize{
        match self {
            Тип::ПримТип(тип) => тип.размер(),
            Тип::Массив {прим_тип, размер} => прим_тип.размер() * размер,
            Тип::Срез {..} => 16, // Два 64-х битных числа: указатель на начало и размер.
        }
    }
}

pub fn проверить_типы(лок: &Лок, ожидаемый_тип: &Тип, действительный_тип: &Тип) -> Результат<()> {
    if ожидаемый_тип == действительный_тип {
        Ok(())
    } else {
        диагностика!(лок, "ОШИБКА", "Несоответствие типов данных. Ожидался тип «{ожидаемый}», но повстречался тип «{действительный}»",
                     ожидаемый = ожидаемый_тип.текст(),
                     действительный = действительный_тип.текст());
        Err(())
    }
}
