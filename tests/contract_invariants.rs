use crabcamera::invariant_ppt;
use crabcamera::types::{CameraFrame, CameraFormat};

#[test]
#[should_panic(expected = "Focus stack frames must have identical dimensions")]
fn contract_focus_stack_invariants() {
    invariant_ppt::clear_invariant_log();
    
    // Setup frames with mismatched dimensions
    let f1 = CameraFrame::new(vec![], 1920, 1080, "dev0".into());
    let f2 = CameraFrame::new(vec![], 1280, 720, "dev0".into()); // Mismatch!
    
    let frames = vec![f1, f2];
    
    // Call merge (should fail, but critically... check invariants)
    // This will Panic because of the assert_invariant! macro
    let _ = crabcamera::focus_stack::merge::merge_frames(&frames, 0.5, 0);
}

#[test]
fn contract_quality_report_invariants() {
    invariant_ppt::clear_invariant_log();
    
    // Construct valid score
    let _score = crabcamera::quality::QualityScore::new(0.5, 0.5, 0.5, 0.5);
    
    invariant_ppt::contract_test("Valid Quality Score", &[
        "Quality components must be normalized 0.0-1.0"
    ]);
}
