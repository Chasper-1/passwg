#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct Avx2Mapper;

impl Avx2Mapper {
    #[target_feature(enable = "avx2")]
    pub unsafe fn map_64_symbols(random_data: *const u8, output_ptr: *mut u8) {
        unsafe {
            // 1. Загружаем 32 байта рандома
            let r = _mm256_loadu_si256(random_data as *const __m256i);

            // 2. Индексы 0-63
            let indices = _mm256_and_si256(r, _mm256_set1_epi8(0x3F));

            // 3. Таблицы символов
            let t0 = _mm256_setr_epi8(65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80, 65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80);
            let t1 = _mm256_setr_epi8(81,82,83,84,85,86,87,88,89,90,97,98,99,100,101,102, 81,82,83,84,85,86,87,88,89,90,97,98,99,100,101,102);
            let t2 = _mm256_setr_epi8(103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118, 103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118);
            let t3 = _mm256_setr_epi8(119,120,121,122,48,49,50,51,52,53,54,55,56,57,95,45, 119,120,121,122,48,49,50,51,52,53,54,55,56,57,95,45);

            // 4. Маппинг
            let low_indices = _mm256_and_si256(indices, _mm256_set1_epi8(0x0F));
            
            let res0 = _mm256_shuffle_epi8(t0, low_indices);
            let res1 = _mm256_shuffle_epi8(t1, low_indices);
            let res2 = _mm256_shuffle_epi8(t2, low_indices);
            let res3 = _mm256_shuffle_epi8(t3, low_indices);

            let mask1 = _mm256_cmpeq_epi8(_mm256_and_si256(indices, _mm256_set1_epi8(0x10)), _mm256_set1_epi8(0x10));
            let blend01 = _mm256_blendv_epi8(res0, res1, mask1);
            
            let mask2 = _mm256_cmpeq_epi8(_mm256_and_si256(indices, _mm256_set1_epi8(0x10)), _mm256_set1_epi8(0x10));
            let blend23 = _mm256_blendv_epi8(res2, res3, mask2);
            
            let mask_final = _mm256_cmpeq_epi8(_mm256_and_si256(indices, _mm256_set1_epi8(0x20)), _mm256_set1_epi8(0x20));
            let final_res = _mm256_blendv_epi8(blend01, blend23, mask_final);

            _mm256_storeu_si256(output_ptr as *mut __m256i, final_res);
        }
    }
}