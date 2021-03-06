use std::cmp::Eq;
use std::fmt;

use crate::geometry::{Point, Vector};
use crate::physics::field::{BHField, Field};
use crate::util::write::DataWriter;

pub mod force;
pub mod barneshut;
pub mod field;

// Mass //////////////////////////////////////////////////////////////////////
//
// Simple wrapper type that can only hold a positive floating point value.

#[derive(PartialEq, Copy, Clone)]
pub struct Mass(f32);

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}

impl From<f32> for Mass {
    fn from(m: f32) -> Self {
        Mass::new(m)
    }
}

impl Mass {
    pub fn new(m: f32) -> Mass {
        if m <= 0.0 { panic!("A mass must be greater than 0. Got {}", m); }
        Mass(m)
    }

    pub fn value(self) -> f32 {
        self.0
    }
}

// Environment ///////////////////////////////////////////////////////////////
//
// An environment represents a space in which bodies interact with fields.

pub struct Environment {
    pub bodies: Vec<Body>,
    pub fields: Vec<Box<dyn Field>>,
    writer: DataWriter,
}

impl Default for Environment {
    fn default() -> Self {
        let field = BHField::new();
        Environment {
            bodies: vec![],
            fields: vec![Box::from(field)],
            writer: DataWriter::new("data"),
        }
    }
}

impl Environment {
    pub fn new(fields: Vec<Box<dyn Field>>, writer: DataWriter) -> Environment {
        Environment { fields, writer, ..Self::default() }
    }

    pub fn update(&mut self) {
        for field in self.fields.iter() {
            let forces = field.forces(&self.bodies[..]);

            for (body, force) in self.bodies.iter_mut().zip(forces.iter()) {
                body.apply_force(force);
            }
        }

        for body in self.bodies.iter_mut() {
            body.apply_velocity();
        }

        let points = self.bodies.iter().map(|b| b.position.clone()).collect();
        self.writer.write(points);
    }
}

// Body //////////////////////////////////////////////////////////////////////
//
// A body represents a movable object in space.

#[derive(Debug)]
pub struct Body {
    pub mass: Mass,
    pub position: Point,
    pub velocity: Vector,
}

impl Clone for Body {
    fn clone(&self) -> Self {
        Body:: new(self.mass.value(), self.position.clone(), self.velocity.clone())
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<>) -> Result<(), fmt::Error> {
        write!(f, "M({}) P({}, {}) V({}, {})",
               self.mass,
               self.position.x, self.position.y,
               self.velocity.dx, self.velocity.dy)
    }
}

impl Eq for Body {}

impl PartialEq for Body {
    fn eq(&self, other: &'_ Body) -> bool {
        // compared referentially
        self as *const _ == other as *const _
    }
}

impl Body {
    pub fn new(mass: f32, position: Point, velocity: Vector) -> Body {
        Body {
            mass: Mass::from(mass),
            position,
            velocity,
        }
    }

    pub fn apply_force(&mut self, force: &Vector) {
        self.velocity += force / self.mass.value();
    }

    pub fn apply_velocity(&mut self) {
        self.position.x += self.velocity.dx;
        self.position.y += self.velocity.dy;
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::geometry::{Point, Vector};

    use super::*;

    #[test]
    #[should_panic(expected = "A mass must be greater than 0.")]
    fn body_with_zero_mass() {
        // given
        Body::new(0.0, Point::zero(), Vector::zero());
    }

    #[test]
    #[should_panic(expected = "A mass must be greater than 0.")]
    fn body_with_negative_mass() {
        // given
        Body::new(-10.0, Point::zero(), Vector::zero());
    }

    #[test]
    fn body_has_referential_equivalence() {
        // given
        let b1 = Body::new(1.0, Point::new(1.0, 2.0), Vector::zero());
        let b2 = b1.clone();

        // then
        assert_eq!(b1, b1);
        assert_ne!(b1, b2);
    }

    #[test]
    fn body_applies_force() {
        // given
        let mut sut = Body::new(2.0, Point::new(1.0, 2.0), Vector::new(-2.0, 5.0));
        let force = Vector { dx: 3.0, dy: -3.0 };

        // when
        sut.apply_force(&force);

        // then
        assert_eq!(Vector::new(-0.5, 3.5), sut.velocity);
        assert_eq!(Point::new(1.0, 2.0), sut.position);
    }

    #[test]
    fn body_applies_velocity() {
        // given
        let mut sut = Body::new(2.0, Point::new(1.0, 2.0), Vector::new(-2.0, 5.0));

        // when
        sut.apply_velocity();

        // then
        assert_eq!(Point::new(-1.0, 7.0), sut.position);
    }
}
