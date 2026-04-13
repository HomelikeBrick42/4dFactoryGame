mod ga;
mod vectors;

pub use ga::{NoE2Rotor, Rotor, Transform};
pub use vectors::*;

mod for_shader {
    ga_generator::ga! {
        element_type = f32;
        scalar_name = s;
        elements = [e0 = zero, e1 = positive_one, e2 = positive_one, e3 = positive_one, e4 = positive_one];

        group Hyperplane = e1 + e2 + e3 + e4;
        group Plane = Hyperplane ^ Hyperplane;
        group Line = Plane ^ Hyperplane;

        group IdealPoint = Line ^ e0;

        fn hyperplane_from_points_abcd(b: IdealPoint, c: IdealPoint, d: IdealPoint) -> Hyperplane {
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            let a = origin;
            let b = origin + b;
            let c = origin + c;
            let d = origin + d;
            return (((a & b) & c) & d);
        }

        fn tetrahedron_side_abc_from_points_abcd(b: IdealPoint, c: IdealPoint, d: IdealPoint) -> Hyperplane {
            let origin = ((e1 ^ e2) ^ e3) ^ e4;
            let a = origin;
            let b = origin + b;
            let c = origin + c;
            let d = origin + d;
            return (((a & b) & c) & d) | ((a & b) & c);
        }
    }
}
