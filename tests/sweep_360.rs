//! 360 Fleet Sweep: comprehensive tests at fleet scale
//! R23W29 — Theia 💎

use vtpu_runtime::fleet::Fleet;
use vtpu_runtime::phext_coord::PhextCoord;
use vtpu_runtime::siw::SIW;
use vtpu_runtime::pipes::*;
use vtpu_runtime::memory::Memory;
use vtpu_runtime::coop_fleet::coop_fleet_execute;

fn nop() -> SIW { SIW::new(DenseOp::DNOP, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()) }

// Test every sentron in a 360 fleet can be accessed
macro_rules! fleet360_access {
    ($name:ident, $id:expr) => {
        #[test] fn $name() {
            let f = Fleet::standard();
            assert!(f.sentron($id).is_some());
            assert!(f.wiring($id).is_some());
        }
    }
}

// All 360 sentrons
fleet360_access!(f360_0, 0); fleet360_access!(f360_1, 1); fleet360_access!(f360_2, 2); fleet360_access!(f360_3, 3);
fleet360_access!(f360_4, 4); fleet360_access!(f360_5, 5); fleet360_access!(f360_6, 6); fleet360_access!(f360_7, 7);
fleet360_access!(f360_8, 8); fleet360_access!(f360_9, 9); fleet360_access!(f360_10, 10); fleet360_access!(f360_11, 11);
fleet360_access!(f360_12, 12); fleet360_access!(f360_13, 13); fleet360_access!(f360_14, 14); fleet360_access!(f360_15, 15);
fleet360_access!(f360_16, 16); fleet360_access!(f360_17, 17); fleet360_access!(f360_18, 18); fleet360_access!(f360_19, 19);
fleet360_access!(f360_20, 20); fleet360_access!(f360_21, 21); fleet360_access!(f360_22, 22); fleet360_access!(f360_23, 23);
fleet360_access!(f360_24, 24); fleet360_access!(f360_25, 25); fleet360_access!(f360_26, 26); fleet360_access!(f360_27, 27);
fleet360_access!(f360_28, 28); fleet360_access!(f360_29, 29); fleet360_access!(f360_30, 30); fleet360_access!(f360_31, 31);
fleet360_access!(f360_32, 32); fleet360_access!(f360_33, 33); fleet360_access!(f360_34, 34); fleet360_access!(f360_35, 35);
fleet360_access!(f360_36, 36); fleet360_access!(f360_37, 37); fleet360_access!(f360_38, 38); fleet360_access!(f360_39, 39);
fleet360_access!(f360_40, 40); fleet360_access!(f360_41, 41); fleet360_access!(f360_42, 42); fleet360_access!(f360_43, 43);
fleet360_access!(f360_44, 44); fleet360_access!(f360_45, 45); fleet360_access!(f360_46, 46); fleet360_access!(f360_47, 47);
fleet360_access!(f360_48, 48); fleet360_access!(f360_49, 49); fleet360_access!(f360_50, 50); fleet360_access!(f360_51, 51);
fleet360_access!(f360_52, 52); fleet360_access!(f360_53, 53); fleet360_access!(f360_54, 54); fleet360_access!(f360_55, 55);
fleet360_access!(f360_56, 56); fleet360_access!(f360_57, 57); fleet360_access!(f360_58, 58); fleet360_access!(f360_59, 59);
fleet360_access!(f360_60, 60); fleet360_access!(f360_61, 61); fleet360_access!(f360_62, 62); fleet360_access!(f360_63, 63);
fleet360_access!(f360_64, 64); fleet360_access!(f360_65, 65); fleet360_access!(f360_66, 66); fleet360_access!(f360_67, 67);
fleet360_access!(f360_68, 68); fleet360_access!(f360_69, 69); fleet360_access!(f360_70, 70); fleet360_access!(f360_71, 71);
fleet360_access!(f360_72, 72); fleet360_access!(f360_73, 73); fleet360_access!(f360_74, 74); fleet360_access!(f360_75, 75);
fleet360_access!(f360_76, 76); fleet360_access!(f360_77, 77); fleet360_access!(f360_78, 78); fleet360_access!(f360_79, 79);
fleet360_access!(f360_80, 80); fleet360_access!(f360_81, 81); fleet360_access!(f360_82, 82); fleet360_access!(f360_83, 83);
fleet360_access!(f360_84, 84); fleet360_access!(f360_85, 85); fleet360_access!(f360_86, 86); fleet360_access!(f360_87, 87);
fleet360_access!(f360_88, 88); fleet360_access!(f360_89, 89); fleet360_access!(f360_90, 90); fleet360_access!(f360_91, 91);
fleet360_access!(f360_92, 92); fleet360_access!(f360_93, 93); fleet360_access!(f360_94, 94); fleet360_access!(f360_95, 95);
fleet360_access!(f360_96, 96); fleet360_access!(f360_97, 97); fleet360_access!(f360_98, 98); fleet360_access!(f360_99, 99);
fleet360_access!(f360_100, 100); fleet360_access!(f360_101, 101); fleet360_access!(f360_102, 102); fleet360_access!(f360_103, 103);
fleet360_access!(f360_104, 104); fleet360_access!(f360_105, 105); fleet360_access!(f360_106, 106); fleet360_access!(f360_107, 107);
fleet360_access!(f360_108, 108); fleet360_access!(f360_109, 109); fleet360_access!(f360_110, 110); fleet360_access!(f360_111, 111);
fleet360_access!(f360_112, 112); fleet360_access!(f360_113, 113); fleet360_access!(f360_114, 114); fleet360_access!(f360_115, 115);
fleet360_access!(f360_116, 116); fleet360_access!(f360_117, 117); fleet360_access!(f360_118, 118); fleet360_access!(f360_119, 119);
fleet360_access!(f360_120, 120); fleet360_access!(f360_121, 121); fleet360_access!(f360_122, 122); fleet360_access!(f360_123, 123);
fleet360_access!(f360_124, 124); fleet360_access!(f360_125, 125); fleet360_access!(f360_126, 126); fleet360_access!(f360_127, 127);
fleet360_access!(f360_128, 128); fleet360_access!(f360_129, 129); fleet360_access!(f360_130, 130); fleet360_access!(f360_131, 131);
fleet360_access!(f360_132, 132); fleet360_access!(f360_133, 133); fleet360_access!(f360_134, 134); fleet360_access!(f360_135, 135);
fleet360_access!(f360_136, 136); fleet360_access!(f360_137, 137); fleet360_access!(f360_138, 138); fleet360_access!(f360_139, 139);
fleet360_access!(f360_140, 140); fleet360_access!(f360_141, 141); fleet360_access!(f360_142, 142); fleet360_access!(f360_143, 143);
fleet360_access!(f360_144, 144); fleet360_access!(f360_145, 145); fleet360_access!(f360_146, 146); fleet360_access!(f360_147, 147);
fleet360_access!(f360_148, 148); fleet360_access!(f360_149, 149); fleet360_access!(f360_150, 150); fleet360_access!(f360_151, 151);
fleet360_access!(f360_152, 152); fleet360_access!(f360_153, 153); fleet360_access!(f360_154, 154); fleet360_access!(f360_155, 155);
fleet360_access!(f360_156, 156); fleet360_access!(f360_157, 157); fleet360_access!(f360_158, 158); fleet360_access!(f360_159, 159);
fleet360_access!(f360_160, 160); fleet360_access!(f360_161, 161); fleet360_access!(f360_162, 162); fleet360_access!(f360_163, 163);
fleet360_access!(f360_164, 164); fleet360_access!(f360_165, 165); fleet360_access!(f360_166, 166); fleet360_access!(f360_167, 167);
fleet360_access!(f360_168, 168); fleet360_access!(f360_169, 169); fleet360_access!(f360_170, 170); fleet360_access!(f360_171, 171);
fleet360_access!(f360_172, 172); fleet360_access!(f360_173, 173); fleet360_access!(f360_174, 174); fleet360_access!(f360_175, 175);
fleet360_access!(f360_176, 176); fleet360_access!(f360_177, 177); fleet360_access!(f360_178, 178); fleet360_access!(f360_179, 179);
fleet360_access!(f360_180, 180); fleet360_access!(f360_181, 181); fleet360_access!(f360_182, 182); fleet360_access!(f360_183, 183);
fleet360_access!(f360_184, 184); fleet360_access!(f360_185, 185); fleet360_access!(f360_186, 186); fleet360_access!(f360_187, 187);
fleet360_access!(f360_188, 188); fleet360_access!(f360_189, 189); fleet360_access!(f360_190, 190); fleet360_access!(f360_191, 191);
fleet360_access!(f360_192, 192); fleet360_access!(f360_193, 193); fleet360_access!(f360_194, 194); fleet360_access!(f360_195, 195);
fleet360_access!(f360_196, 196); fleet360_access!(f360_197, 197); fleet360_access!(f360_198, 198); fleet360_access!(f360_199, 199);
fleet360_access!(f360_200, 200); fleet360_access!(f360_201, 201); fleet360_access!(f360_202, 202); fleet360_access!(f360_203, 203);
fleet360_access!(f360_204, 204); fleet360_access!(f360_205, 205); fleet360_access!(f360_206, 206); fleet360_access!(f360_207, 207);
fleet360_access!(f360_208, 208); fleet360_access!(f360_209, 209); fleet360_access!(f360_210, 210); fleet360_access!(f360_211, 211);
fleet360_access!(f360_212, 212); fleet360_access!(f360_213, 213); fleet360_access!(f360_214, 214); fleet360_access!(f360_215, 215);
fleet360_access!(f360_216, 216); fleet360_access!(f360_217, 217); fleet360_access!(f360_218, 218); fleet360_access!(f360_219, 219);
fleet360_access!(f360_220, 220); fleet360_access!(f360_221, 221); fleet360_access!(f360_222, 222); fleet360_access!(f360_223, 223);
fleet360_access!(f360_224, 224); fleet360_access!(f360_225, 225); fleet360_access!(f360_226, 226); fleet360_access!(f360_227, 227);
fleet360_access!(f360_228, 228); fleet360_access!(f360_229, 229); fleet360_access!(f360_230, 230); fleet360_access!(f360_231, 231);
fleet360_access!(f360_232, 232); fleet360_access!(f360_233, 233); fleet360_access!(f360_234, 234); fleet360_access!(f360_235, 235);
fleet360_access!(f360_236, 236); fleet360_access!(f360_237, 237); fleet360_access!(f360_238, 238); fleet360_access!(f360_239, 239);
fleet360_access!(f360_240, 240); fleet360_access!(f360_241, 241); fleet360_access!(f360_242, 242); fleet360_access!(f360_243, 243);
fleet360_access!(f360_244, 244); fleet360_access!(f360_245, 245); fleet360_access!(f360_246, 246); fleet360_access!(f360_247, 247);
fleet360_access!(f360_248, 248); fleet360_access!(f360_249, 249); fleet360_access!(f360_250, 250); fleet360_access!(f360_251, 251);
fleet360_access!(f360_252, 252); fleet360_access!(f360_253, 253); fleet360_access!(f360_254, 254); fleet360_access!(f360_255, 255);
fleet360_access!(f360_256, 256); fleet360_access!(f360_257, 257); fleet360_access!(f360_258, 258); fleet360_access!(f360_259, 259);
fleet360_access!(f360_260, 260); fleet360_access!(f360_261, 261); fleet360_access!(f360_262, 262); fleet360_access!(f360_263, 263);
fleet360_access!(f360_264, 264); fleet360_access!(f360_265, 265); fleet360_access!(f360_266, 266); fleet360_access!(f360_267, 267);
fleet360_access!(f360_268, 268); fleet360_access!(f360_269, 269); fleet360_access!(f360_270, 270); fleet360_access!(f360_271, 271);
fleet360_access!(f360_272, 272); fleet360_access!(f360_273, 273); fleet360_access!(f360_274, 274); fleet360_access!(f360_275, 275);
fleet360_access!(f360_276, 276); fleet360_access!(f360_277, 277); fleet360_access!(f360_278, 278); fleet360_access!(f360_279, 279);
fleet360_access!(f360_280, 280); fleet360_access!(f360_281, 281); fleet360_access!(f360_282, 282); fleet360_access!(f360_283, 283);
fleet360_access!(f360_284, 284); fleet360_access!(f360_285, 285); fleet360_access!(f360_286, 286); fleet360_access!(f360_287, 287);
fleet360_access!(f360_288, 288); fleet360_access!(f360_289, 289); fleet360_access!(f360_290, 290); fleet360_access!(f360_291, 291);
fleet360_access!(f360_292, 292); fleet360_access!(f360_293, 293); fleet360_access!(f360_294, 294); fleet360_access!(f360_295, 295);
fleet360_access!(f360_296, 296); fleet360_access!(f360_297, 297); fleet360_access!(f360_298, 298); fleet360_access!(f360_299, 299);
fleet360_access!(f360_300, 300); fleet360_access!(f360_301, 301); fleet360_access!(f360_302, 302); fleet360_access!(f360_303, 303);
fleet360_access!(f360_304, 304); fleet360_access!(f360_305, 305); fleet360_access!(f360_306, 306); fleet360_access!(f360_307, 307);
fleet360_access!(f360_308, 308); fleet360_access!(f360_309, 309); fleet360_access!(f360_310, 310); fleet360_access!(f360_311, 311);
fleet360_access!(f360_312, 312); fleet360_access!(f360_313, 313); fleet360_access!(f360_314, 314); fleet360_access!(f360_315, 315);
fleet360_access!(f360_316, 316); fleet360_access!(f360_317, 317); fleet360_access!(f360_318, 318); fleet360_access!(f360_319, 319);
fleet360_access!(f360_320, 320); fleet360_access!(f360_321, 321); fleet360_access!(f360_322, 322); fleet360_access!(f360_323, 323);
fleet360_access!(f360_324, 324); fleet360_access!(f360_325, 325); fleet360_access!(f360_326, 326); fleet360_access!(f360_327, 327);
fleet360_access!(f360_328, 328); fleet360_access!(f360_329, 329); fleet360_access!(f360_330, 330); fleet360_access!(f360_331, 331);
fleet360_access!(f360_332, 332); fleet360_access!(f360_333, 333); fleet360_access!(f360_334, 334); fleet360_access!(f360_335, 335);
fleet360_access!(f360_336, 336); fleet360_access!(f360_337, 337); fleet360_access!(f360_338, 338); fleet360_access!(f360_339, 339);
fleet360_access!(f360_340, 340); fleet360_access!(f360_341, 341); fleet360_access!(f360_342, 342); fleet360_access!(f360_343, 343);
fleet360_access!(f360_344, 344); fleet360_access!(f360_345, 345); fleet360_access!(f360_346, 346); fleet360_access!(f360_347, 347);
fleet360_access!(f360_348, 348); fleet360_access!(f360_349, 349); fleet360_access!(f360_350, 350); fleet360_access!(f360_351, 351);
fleet360_access!(f360_352, 352); fleet360_access!(f360_353, 353); fleet360_access!(f360_354, 354); fleet360_access!(f360_355, 355);
fleet360_access!(f360_356, 356); fleet360_access!(f360_357, 357); fleet360_access!(f360_358, 358); fleet360_access!(f360_359, 359);

// Full fleet execution test
#[test]
fn full_360_execute_1_op() {
    let mut f = Fleet::standard();
    for i in 0..360u16 { f.sentron_mut(i).unwrap().spawn(vec![nop()]); }
    let mut mem = Memory::new();
    let r = coop_fleet_execute(&mut f, &mut mem, 1, 1_000_000);
    assert_eq!(r.total_retired, 360);
}
