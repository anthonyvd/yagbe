use crate::framebuffer::Framebuffer;

pub struct FramebufferBundle {
  pub main: Framebuffer,
  pub tiles: Framebuffer,
}